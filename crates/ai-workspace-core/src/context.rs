use crate::error::Result;
use crate::paths::WorkspacePaths;
use crate::search::{self, SourceKind};
use crate::workspace::{
    Project, active_temporary_paths, collect_markdown_files, looks_like_secret, scan_markdown_files,
};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_CONTEXT_SOURCES: usize = 8;
pub const DEFAULT_MAX_CONTEXT_BYTES: usize = 12000;

/// One extra source is allowed per this many bytes of budget, between the default 8 and 32.
const BYTES_PER_EXTRA_SOURCE: usize = 1500;
const MAX_CONTEXT_SOURCES: usize = 32;
/// A file is cut rather than left out only if at least this much of it would fit.
const MIN_TRUNCATED_BYTES: usize = 512;

/// How much workspace text one context may hold (ADR-0012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextBudget {
    pub max_sources: usize,
    pub max_bytes: usize,
    /// The model's context window in tokens this budget was derived from, if any.
    pub window_tokens: Option<u32>,
}

impl Default for ContextBudget {
    /// The fixed budget used when the model's window is unknown.
    fn default() -> Self {
        Self {
            max_sources: DEFAULT_MAX_CONTEXT_SOURCES,
            max_bytes: DEFAULT_MAX_CONTEXT_BYTES,
            window_tokens: None,
        }
    }
}

impl ContextBudget {
    /// A budget for a model with a `window_tokens` context window, leaving room for
    /// `reserved_bytes` of other system text (the model's own instructions). Workspace text
    /// gets 1.5 bytes per window token: about 40 % of the window at roughly 3.75 bytes per
    /// token, leaving the rest for instructions, the conversation and the reply.
    pub fn for_window(window_tokens: u32, reserved_bytes: usize) -> Self {
        let window_bytes = window_tokens as usize * 3 / 2;
        let max_bytes = window_bytes.saturating_sub(reserved_bytes);
        Self {
            max_sources: (max_bytes / BYTES_PER_EXTRA_SOURCE)
                .clamp(DEFAULT_MAX_CONTEXT_SOURCES, MAX_CONTEXT_SOURCES),
            max_bytes,
            window_tokens: Some(window_tokens),
        }
    }
}

impl std::fmt::Display for ContextBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.window_tokens {
            Some(tokens) => write!(
                f,
                "{} bytes of workspace text, from the model's {tokens}-token window",
                self.max_bytes
            ),
            None => write!(
                f,
                "{} bytes of workspace text (the model's window is unknown)",
                self.max_bytes
            ),
        }
    }
}

impl std::fmt::Display for TruncatedFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} of {} bytes shown)",
            self.path.display(),
            self.shown,
            self.total
        )
    }
}

/// A file only partly included because it did not fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncatedFile {
    pub path: PathBuf,
    /// Bytes included.
    pub shown: usize,
    /// The file's full size in bytes.
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub base_files: Vec<PathBuf>,
    pub project_files: Vec<PathBuf>,
    pub session_files: Vec<PathBuf>,
    /// Files from the session's temporary grants (ADR-0014).
    pub granted_files: Vec<PathBuf>,
    pub assembled_content: String,
    pub source_count: usize,
    pub total_bytes: usize,
    /// Files and folders left out because their names look like secrets (ADR-0012).
    pub skipped_files: Vec<PathBuf>,
    /// Files included only in part, each marked in the text (ADR-0012).
    pub truncated_files: Vec<TruncatedFile>,
    /// The budget this context was built within.
    pub budget: ContextBudget,
}

impl ContextSnapshot {
    pub fn as_system_prompt(&self) -> String {
        format!(
            "You are assisting inside the AI Workspace tool. Use the following workspace context to answer the user's questions. If the context does not cover something, say so instead of guessing.\n{}",
            self.assembled_content
        )
    }

    /// The system message for a model with its own instructions (ADR-0012): the model's
    /// `SYSTEM` text first, unchanged, then the workspace context. Ollama uses a request's
    /// system message in place of the Modelfile's, so leaving the instructions out would
    /// drop the agent's rules. Without instructions this is [`Self::as_system_prompt`].
    pub fn as_system_prompt_for(&self, agent_instructions: Option<&str>) -> String {
        match agent_instructions {
            Some(instructions) => format!("{instructions}\n\n---\n\n{}", self.as_system_prompt()),
            None => self.as_system_prompt(),
        }
    }
}

pub fn build_context_snapshot(
    paths: &WorkspacePaths,
    project: Option<&Project>,
    query: Option<&str>,
) -> Result<ContextSnapshot> {
    build_context_snapshot_within(paths, project, query, ContextBudget::default())
}

/// Like [`build_context_snapshot`], within `budget`. Sources are taken in order while they fit;
/// the first one that does not fit is cut, with a visible
/// `[truncated: N of M bytes shown]` marker, if at least a useful part of it fits.
pub fn build_context_snapshot_within(
    paths: &WorkspacePaths,
    project: Option<&Project>,
    query: Option<&str>,
    budget: ContextBudget,
) -> Result<ContextSnapshot> {
    let readable = readable_sources(paths, project)?;
    let skipped_files = readable.skipped;
    let all_sources: Vec<(String, PathBuf)> = readable
        .sources
        .into_iter()
        .map(|(kind, path)| (kind.as_str().to_string(), path))
        .collect();

    let ordered_sources: Vec<(String, PathBuf)> = match query {
        Some(text) => {
            let candidates: Vec<search::SearchCandidate> = all_sources
                .iter()
                .map(|(kind, path)| search::SearchCandidate {
                    source_kind: source_kind_from_tag(kind),
                    path: path.clone(),
                })
                .collect();

            search::order_by_relevance(&candidates, text)
                .into_iter()
                .map(|result| (result.source_kind.as_str().to_string(), result.path))
                .collect()
        }
        None => {
            let mut sources = all_sources;
            sources.sort_by(|left, right| {
                let rank = |kind: &str| match kind {
                    "project" => 0,
                    "base" => 1,
                    "session" => 2,
                    "granted" => 3,
                    _ => 4,
                };
                rank(left.0.as_str())
                    .cmp(&rank(right.0.as_str()))
                    .then_with(|| left.1.cmp(&right.1))
            });
            sources
        }
    };

    let mut selected_sources = Vec::new();
    let mut assembled_content = String::new();
    let mut total_bytes = 0usize;
    let mut truncated_files = Vec::new();

    for (kind, path) in ordered_sources.into_iter() {
        if selected_sources.len() >= budget.max_sources {
            break;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let bytes = content.len();
        let room = budget.max_bytes.saturating_sub(total_bytes);
        if bytes > room {
            // Cut only once, and only when a useful part fits; smaller files later in the
            // order may still fit whole, as before.
            if !truncated_files.is_empty() || room < MIN_TRUNCATED_BYTES {
                continue;
            }
            let shown_text = cut_at_line(&content, room);
            let shown = shown_text.len();
            assembled_content.push_str(&format!("\n--- {} ---\n", path.display()));
            assembled_content.push_str(shown_text);
            assembled_content.push_str(&format!("\n[truncated: {shown} of {bytes} bytes shown]\n"));
            selected_sources.push((kind.clone(), path.clone()));
            total_bytes += shown;
            truncated_files.push(TruncatedFile {
                path: path.clone(),
                shown,
                total: bytes,
            });
            continue;
        }

        selected_sources.push((kind.clone(), path.clone()));
        total_bytes += bytes;
        assembled_content.push_str(&format!("\n--- {} ---\n", path.display()));
        assembled_content.push_str(&content);
        assembled_content.push('\n');
    }

    let selected_base = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "base")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let selected_project = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "project")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let selected_session = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "session")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let selected_granted = selected_sources
        .iter()
        .filter(|(kind, _)| kind == "granted")
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();

    Ok(ContextSnapshot {
        base_files: selected_base,
        project_files: selected_project,
        session_files: selected_session,
        granted_files: selected_granted,
        assembled_content,
        source_count: selected_sources.len(),
        total_bytes,
        skipped_files,
        truncated_files,
        budget,
    })
}

fn source_kind_from_tag(kind: &str) -> SourceKind {
    match kind {
        "project" => SourceKind::Project,
        "session" => SourceKind::Session,
        "granted" => SourceKind::Granted,
        _ => SourceKind::Base,
    }
}

/// How deep below Base and the project folder Markdown is looked for: the folder itself, its
/// subfolders, and one level below those (e.g. `Projects/sec/Knowledge/router/notes.md`).
const DEFAULT_SCOPE_DEPTH: usize = 3;
/// How deep below a granted folder Markdown is looked for.
const GRANTED_FOLDER_DEPTH: usize = 2;

/// What a context or a search may read (ADR-0014), with the files the block list left out.
#[derive(Debug, Clone, Default)]
pub struct ReadableSources {
    /// Every readable Markdown file, with what kind of source it is. No path appears twice.
    pub sources: Vec<(SourceKind, PathBuf)>,
    /// Files and folders skipped because their names look like secrets (ADR-0012).
    pub skipped: Vec<PathBuf>,
}

/// The one definition of what the model may read: Markdown in all of Base and, with a project,
/// the whole project folder (its `Session/` folder as session sources), plus the project's
/// active temporary grants, each a file or a folder. The secret block list applies to all of it;
/// a grant inside Base or the project adds nothing new.
pub fn readable_sources(
    paths: &WorkspacePaths,
    project: Option<&Project>,
) -> Result<ReadableSources> {
    let mut readable = ReadableSources::default();
    let mut seen = std::collections::HashSet::new();
    let mut add = |readable: &mut ReadableSources, kind: SourceKind, path: PathBuf| {
        if seen.insert(path.clone()) {
            readable.sources.push((kind, path));
        }
    };

    let base = paths.base_knowledge_path();
    let scan = scan_markdown_files(base, DEFAULT_SCOPE_DEPTH)?;
    readable
        .skipped
        .extend(scan.skipped.iter().map(|p| base.join(p)));
    for file in scan.files {
        add(&mut readable, SourceKind::Base, base.join(file));
    }

    let Some(project) = project else {
        return Ok(readable);
    };

    let scan = scan_markdown_files(&project.path, DEFAULT_SCOPE_DEPTH).unwrap_or_default();
    readable
        .skipped
        .extend(scan.skipped.iter().map(|p| project.path.join(p)));
    let session_root = project.path.join("Session");
    for file in scan.files {
        let path = project.path.join(file);
        let kind = if path.starts_with(&session_root) {
            SourceKind::Session
        } else {
            SourceKind::Project
        };
        add(&mut readable, kind, path);
    }

    for granted in active_temporary_paths(paths, project)? {
        if granted.starts_with(base) || granted.starts_with(&project.path) {
            continue;
        }
        if granted.is_dir() {
            let scan = scan_markdown_files(&granted, GRANTED_FOLDER_DEPTH).unwrap_or_default();
            readable
                .skipped
                .extend(scan.skipped.iter().map(|p| granted.join(p)));
            for file in scan.files {
                add(&mut readable, SourceKind::Granted, granted.join(file));
            }
        } else if granted.extension().is_some_and(|ext| ext == "md") {
            let name = granted
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            if looks_like_secret(&name) {
                readable.skipped.push(granted);
            } else {
                add(&mut readable, SourceKind::Granted, granted);
            }
        }
    }

    readable.skipped.sort();
    readable.skipped.dedup();
    Ok(readable)
}

pub fn collect_context_sources(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    collect_markdown_files(root, max_depth)
}

/// The longest start of `text` that is at most `max_bytes` long, ending at a line break when
/// one is reasonably close, and always on a character boundary.
fn cut_at_line(text: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let head = &text[..end];
    match head.rfind('\n') {
        Some(line_end) if line_end >= end / 2 => &head[..line_end],
        _ => head,
    }
}
