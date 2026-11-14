//! Layer 5 Search Engine (see `ARCHITECTURE.md`): simple, dependency-free
//! filesystem search over the workspace's Markdown knowledge.
//!
//! This module owns scoring, ranking, and snippet extraction only. It never
//! traverses directories or checks permissions itself — callers (`session.rs`)
//! collect an already permission-filtered candidate list and hand it to
//! `rank_candidates`. See `specs/006-filesystem-search/` for the full design.

use std::cmp::Ordering;
use std::path::PathBuf;

const SNIPPET_WINDOW_CHARS: usize = 160;
const FILENAME_MATCH_BONUS: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Base,
    Project,
    Session,
}

impl SourceKind {
    fn priority(self) -> u8 {
        match self {
            SourceKind::Project => 0,
            SourceKind::Base => 1,
            SourceKind::Session => 2,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Base => "base",
            SourceKind::Project => "project",
            SourceKind::Session => "session",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(text: &str, limit: usize) -> Self {
        Self {
            text: text.to_string(),
            limit,
        }
    }

    fn terms(&self) -> Vec<String> {
        self.text
            .split_whitespace()
            .filter(|term| !term.is_empty())
            .map(|term| term.to_ascii_lowercase())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SearchCandidate {
    pub source_kind: SourceKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub source_kind: SourceKind,
    pub path: PathBuf,
    pub relevance_score: f64,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub query_text: String,
    pub results: Vec<SearchResult>,
}

/// Case-insensitive, whole-word term-frequency scoring with a filename-match
/// bonus. No stemming, fuzzy matching, or external ranking library — see
/// `research.md` Decision 1 for why this is enough for v0.5.
fn score_candidate(content: &str, path: &std::path::Path, terms: &[String]) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }

    let words: Vec<String> = content
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect();

    let file_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut score = 0.0;
    for term in terms {
        let occurrences = words.iter().filter(|word| word.as_str() == term).count();
        score += occurrences as f64;
        if !term.is_empty() && file_name.contains(term.as_str()) {
            score += FILENAME_MATCH_BONUS;
        }
    }

    score
}

/// Bounded (~160 char), word-boundary-trimmed excerpt around the first term
/// match, with a leading/trailing ellipsis when truncated. See `research.md`
/// Decision 6.
fn extract_snippet(content: &str, terms: &[String]) -> String {
    let lower_content = content.to_ascii_lowercase();
    let match_byte_index = terms
        .iter()
        .filter(|term| !term.is_empty())
        .filter_map(|term| lower_content.find(term.as_str()))
        .min()
        .unwrap_or(0);

    windowed_snippet(content, match_byte_index)
}

fn windowed_snippet(content: &str, byte_center: usize) -> String {
    let chars: Vec<(usize, char)> = content.char_indices().collect();
    if chars.is_empty() {
        return String::new();
    }

    let center_char_index = chars
        .iter()
        .position(|(byte_index, _)| *byte_index >= byte_center)
        .unwrap_or(chars.len() - 1);

    let half_chars = SNIPPET_WINDOW_CHARS / 2;
    let start_char = center_char_index.saturating_sub(half_chars);
    let end_char = (center_char_index + half_chars).min(chars.len());

    let start_byte = chars[start_char].0;
    let end_byte = if end_char < chars.len() {
        chars[end_char].0
    } else {
        content.len()
    };

    let mut snippet = content[start_byte..end_byte].trim().to_string();

    if start_char > 0 {
        if let Some(space_index) = snippet.find(char::is_whitespace) {
            snippet = snippet[space_index..].trim_start().to_string();
        }
        snippet = format!("...{snippet}");
    }

    if end_byte < content.len() {
        if let Some(space_index) = snippet.rfind(char::is_whitespace) {
            snippet.truncate(space_index);
        }
        snippet = format!("{snippet}...");
    }

    snippet.trim().to_string()
}

/// Scores and sorts every candidate by relevance to `terms`. Unreadable
/// candidates are skipped (FR-009: treat a document that's become
/// unavailable the same as any other missing context source). Whether
/// zero-score candidates survive is controlled by `drop_zero_scores` — the
/// standalone search command wants them gone (a zero-relevance "result" is
/// noise), but the Context Builder wants them kept and merely sorted to the
/// bottom, so a small workspace still gets everything it did in v0.4
/// regardless of relevance (spec.md User Story 2, AC2).
fn ranked_results(
    candidates: &[SearchCandidate],
    terms: &[String],
    drop_zero_scores: bool,
) -> Vec<SearchResult> {
    let mut results: Vec<SearchResult> = candidates
        .iter()
        .filter_map(|candidate| {
            let content = std::fs::read_to_string(&candidate.path).ok()?;
            let score = score_candidate(&content, &candidate.path, terms);
            if drop_zero_scores && score <= 0.0 {
                return None;
            }

            Some(SearchResult {
                source_kind: candidate.source_kind,
                path: candidate.path.clone(),
                relevance_score: score,
                snippet: extract_snippet(&content, terms),
            })
        })
        .collect();

    results.sort_by(|left, right| {
        right
            .relevance_score
            .partial_cmp(&left.relevance_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                left.source_kind
                    .priority()
                    .cmp(&right.source_kind.priority())
            })
            .then_with(|| left.path.cmp(&right.path))
    });

    results
}

/// Pure function: no filesystem traversal, no permission logic. Callers
/// supply an already permission-filtered candidate list (see
/// `contracts/context-builder-integration.md`). Used by the standalone
/// search command — non-matches are dropped and the list is capped to
/// `query.limit`.
pub fn rank_candidates(candidates: &[SearchCandidate], query: &SearchQuery) -> Vec<SearchResult> {
    let mut results = ranked_results(candidates, &query.terms(), true);
    results.truncate(query.limit.max(1));
    results
}

/// Orders every candidate by relevance to `query_text` without dropping
/// zero-score candidates or capping the count. Used by the Context Builder
/// (`context::build_context_snapshot`), whose own source/byte caps do the
/// truncation — relevance only decides *which* sources get cut when there
/// isn't room for everything, not whether an unrelated-but-fitting source is
/// excluded outright.
pub fn order_by_relevance(candidates: &[SearchCandidate], query_text: &str) -> Vec<SearchResult> {
    let terms = SearchQuery::new(query_text, 0).terms();
    ranked_results(candidates, &terms, false)
}

/// Runs `rank_candidates` and wraps the result for display/consumption,
/// echoing the query text back per `data-model.md`'s `SearchOutcome`.
pub fn search(candidates: &[SearchCandidate], query: &SearchQuery) -> SearchOutcome {
    SearchOutcome {
        query_text: query.text.clone(),
        results: rank_candidates(candidates, query),
    }
}
