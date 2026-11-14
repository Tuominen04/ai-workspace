use crate::COMMAND_NAME;
use crate::error::{AppError, GatedOperation, Result};
use crate::paths::WorkspacePaths;
use crate::session;
use crate::ui;

const USAGE_CONFIG_SET_ROOT: &str = "config set-root PATH";
const USAGE_PROJECT_SHOW: &str = "project show PROJECT";
const USAGE_PROJECT_CREATE: &str = "project create --name NAME [--description TEXT]";
const USAGE_PERMISSION_GRANT: &str = "permission grant [--project PROJECT] --path PATH";
const USAGE_PERMISSION_VIEW: &str =
    "permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]";
const USAGE_PERMISSION_REVOKE: &str = "permission revoke [--project PROJECT] --path PATH";
const USAGE_SEARCH: &str = "search --query TEXT [--project PROJECT] [--limit N]";

/// Reports a failed command: whatever the error carries for the user to read
/// on stdout first (the blocking issues of a refused gated command), then
/// `error: ...` on stderr.
pub fn report_error(error: &AppError) {
    if let AppError::ValidationGateBlocked { operation, result } = error {
        ui::render_validation_gate_block(&gated_command(*operation), result);
    }
    eprintln!("error: {}", error_message(error));
}

/// The message for an error, in the CLI's words. Errors whose core wording
/// would not tell a terminal user what to type are reworded here; everything
/// else uses the core's message.
fn error_message(error: &AppError) -> String {
    match error {
        AppError::ValidationGateBlocked { operation, .. } => format!(
            "cannot run {} because workspace validation has blocking issues",
            gated_command(*operation)
        ),
        AppError::MissingProjectContext => "permission command requires a project context; use --project PROJECT or keep only one project".to_string(),
        other => other.to_string(),
    }
}

/// The command line of a gated operation, e.g. `ai-workspace project create`.
fn gated_command(operation: GatedOperation) -> String {
    let subcommand = match operation {
        GatedOperation::Search => "search",
        GatedOperation::ProjectCreate => "project create",
        GatedOperation::ProjectShow => "project show",
        GatedOperation::PermissionGrant => "permission grant",
        GatedOperation::PermissionRevoke => "permission revoke",
        GatedOperation::PermissionView => "permission view",
    };
    format!("{COMMAND_NAME} {subcommand}")
}

/// Builds the usage error for one subcommand, prefixed with the command name.
fn usage_error(usage: &str) -> AppError {
    AppError::Message(format!("usage: {COMMAND_NAME} {usage}"))
}

pub fn run() -> Result<()> {
    let command = parse_command(std::env::args().skip(1).collect())?;

    // These never need a workspace, so they must work even before one is
    // chosen, or when the saved workspace setting is broken.
    match command {
        Command::Version => print_version(),
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::ConfigSetRoot { path } => session::set_workspace_root(&path),
        workspace_command => run_workspace_command(workspace_command),
    }
}

fn run_workspace_command(command: Command) -> Result<()> {
    let (paths, source) = WorkspacePaths::detect_with_source()?;

    match command {
        Command::Run => session::run_session(&paths),
        Command::Init => session::init_workspace(&paths),
        Command::Status => session::print_status(&paths, &source),
        Command::ConfigShow => session::show_workspace_config(&paths, &source),
        Command::ProjectCreate { name, description } => {
            session::create_managed_project(&paths, &name, description.as_deref())
        }
        Command::ProjectShow { project } => session::show_managed_project(&paths, &project),
        Command::PermissionGrant { project, path } => {
            session::grant_permission(&paths, project.as_deref(), &path)
        }
        Command::PermissionView {
            project,
            filter,
            page,
            page_size,
        } => session::view_permissions(
            &paths,
            project.as_deref(),
            filter.as_deref(),
            page,
            page_size,
        ),
        Command::PermissionRevoke { project, path } => {
            session::revoke_permission(&paths, project.as_deref(), &path)
        }
        Command::Search {
            query,
            project,
            limit,
        } => session::run_search(&paths, &query, project.as_deref(), limit),
        Command::Validate => session::validate_workspace_state(&paths),
        // Handled in `run` before a workspace is resolved; never reached.
        Command::Version | Command::Help | Command::ConfigSetRoot { .. } => Ok(()),
    }
}

enum Command {
    Run,
    Init,
    Status,
    ConfigShow,
    ConfigSetRoot {
        path: String,
    },
    ProjectCreate {
        name: String,
        description: Option<String>,
    },
    ProjectShow {
        project: String,
    },
    PermissionGrant {
        project: Option<String>,
        path: String,
    },
    PermissionView {
        project: Option<String>,
        filter: Option<String>,
        page: usize,
        page_size: usize,
    },
    PermissionRevoke {
        project: Option<String>,
        path: String,
    },
    Search {
        query: String,
        project: Option<String>,
        limit: usize,
    },
    Validate,
    Version,
    Help,
}

fn parse_command(args: Vec<String>) -> Result<Command> {
    if args.is_empty() {
        return Ok(Command::Run);
    }

    match args[0].as_str() {
        "init" => Ok(Command::Init),
        "status" => Ok(Command::Status),
        "validate" => Ok(Command::Validate),
        "config" => parse_config_command(&args[1..]),
        "project" => parse_project_command(&args[1..]),
        "permission" => parse_permission_command(&args[1..]),
        "search" => parse_search_command(&args[1..]),
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-V" | "--version" => Ok(Command::Version),
        other => Err(AppError::Message(format!(
            "unknown command '{other}'. Use '{COMMAND_NAME} --help' for usage."
        ))),
    }
}

fn parse_config_command(args: &[String]) -> Result<Command> {
    match args {
        [sub] if sub == "show" => Ok(Command::ConfigShow),
        [sub, path] if sub == "set-root" => Ok(Command::ConfigSetRoot { path: path.clone() }),
        [sub] if sub == "set-root" => Err(usage_error(USAGE_CONFIG_SET_ROOT)),
        _ => Err(AppError::Message(format!(
            "invalid config command. Use '{COMMAND_NAME} config show' or '{COMMAND_NAME} config set-root PATH'."
        ))),
    }
}

fn parse_project_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        return Err(AppError::Message(format!(
            "missing project subcommand. Use '{COMMAND_NAME} project create' or '{COMMAND_NAME} project show'."
        )));
    }

    match args[0].as_str() {
        "show" => {
            if args.len() != 2 {
                return Err(usage_error(USAGE_PROJECT_SHOW));
            }
            Ok(Command::ProjectShow {
                project: args[1].clone(),
            })
        }
        "create" => parse_project_create_command(&args[1..]),
        _ => Err(AppError::Message(format!(
            "invalid project command. Use '{COMMAND_NAME} project create' or '{COMMAND_NAME} project show'."
        ))),
    }
}

fn parse_project_create_command(args: &[String]) -> Result<Command> {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--name" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PROJECT_CREATE));
                }
                name = Some(args[index + 1].clone());
                index += 2;
            }
            "--description" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PROJECT_CREATE));
                }
                description = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(usage_error(USAGE_PROJECT_CREATE));
            }
        }
    }

    let name = name.ok_or_else(|| usage_error(USAGE_PROJECT_CREATE))?;

    Ok(Command::ProjectCreate { name, description })
}

fn parse_permission_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        return Err(AppError::Message(format!(
            "missing permission subcommand. Use '{COMMAND_NAME} permission grant', '{COMMAND_NAME} permission view', or '{COMMAND_NAME} permission revoke'."
        )));
    }

    match args[0].as_str() {
        "grant" => parse_permission_grant_command(&args[1..]),
        "view" => parse_permission_view_command(&args[1..]),
        "revoke" => parse_permission_revoke_command(&args[1..]),
        _ => Err(AppError::Message(format!(
            "invalid permission command. Use '{COMMAND_NAME} permission grant', '{COMMAND_NAME} permission view', or '{COMMAND_NAME} permission revoke'."
        ))),
    }
}

fn parse_permission_grant_command(args: &[String]) -> Result<Command> {
    let mut project: Option<String> = None;
    let mut path: Option<String> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_GRANT));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--path" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_GRANT));
                }
                path = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(usage_error(USAGE_PERMISSION_GRANT));
            }
        }
    }

    let path = path.ok_or_else(|| usage_error(USAGE_PERMISSION_GRANT))?;

    Ok(Command::PermissionGrant { project, path })
}

fn parse_permission_revoke_command(args: &[String]) -> Result<Command> {
    let mut project: Option<String> = None;
    let mut path: Option<String> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_REVOKE));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--path" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_REVOKE));
                }
                path = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(usage_error(USAGE_PERMISSION_REVOKE));
            }
        }
    }

    let path = path.ok_or_else(|| usage_error(USAGE_PERMISSION_REVOKE))?;

    Ok(Command::PermissionRevoke { project, path })
}

fn parse_permission_view_command(args: &[String]) -> Result<Command> {
    let mut project: Option<String> = None;
    let mut filter: Option<String> = None;
    let mut page: usize = 1;
    let mut page_size: usize = 25;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_VIEW));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--filter" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_VIEW));
                }
                filter = Some(args[index + 1].clone());
                index += 2;
            }
            "--page" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_VIEW));
                }
                page = args[index + 1].parse::<usize>().map_err(|_| {
                    AppError::Message("--page must be a positive integer".to_string())
                })?;
                index += 2;
            }
            "--page-size" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_PERMISSION_VIEW));
                }
                page_size = args[index + 1].parse::<usize>().map_err(|_| {
                    AppError::Message("--page-size must be a positive integer".to_string())
                })?;
                index += 2;
            }
            _ => {
                return Err(usage_error(USAGE_PERMISSION_VIEW));
            }
        }
    }

    Ok(Command::PermissionView {
        project,
        filter,
        page,
        page_size,
    })
}

fn parse_search_command(args: &[String]) -> Result<Command> {
    let mut query: Option<String> = None;
    let mut project: Option<String> = None;
    let mut limit: usize = 10;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--query" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_SEARCH));
                }
                query = Some(args[index + 1].clone());
                index += 2;
            }
            "--project" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_SEARCH));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--limit" => {
                if index + 1 >= args.len() {
                    return Err(usage_error(USAGE_SEARCH));
                }
                limit = args[index + 1].parse::<usize>().map_err(|_| {
                    AppError::Message("--limit must be a positive integer".to_string())
                })?;
                if limit == 0 {
                    return Err(AppError::Message(
                        "--limit must be a positive integer".to_string(),
                    ));
                }
                index += 2;
            }
            _ => {
                return Err(usage_error(USAGE_SEARCH));
            }
        }
    }

    let query = match query {
        Some(query) if !query.trim().is_empty() => query,
        _ => return Err(usage_error(USAGE_SEARCH)),
    };

    Ok(Command::Search {
        query,
        project,
        limit,
    })
}

fn print_help() {
    let cmd = COMMAND_NAME;
    // Descriptions on their own line are indented to the description column.
    let pad = " ".repeat(cmd.len() + 13);

    outln!("AI Workspace");
    outln!();
    outln!("Usage:");
    outln!("  {cmd}           Launch an interactive workspace session");
    outln!("  {cmd} init      Create the standard ~/AI directory layout");
    outln!("  {cmd} status    Show discovered workspaces and root paths");
    outln!("  {cmd} validate  Validate workspace and project definitions");
    outln!("  {cmd} config show");
    outln!("{pad}Show workspace configuration");
    outln!("  {cmd} {USAGE_CONFIG_SET_ROOT}");
    outln!("{pad}Remember PATH as the workspace folder (instead of ~/AI)");
    outln!("  {cmd} {USAGE_PROJECT_CREATE}");
    outln!("{pad}Create a managed project workspace");
    outln!("  {cmd} {USAGE_PROJECT_SHOW}");
    outln!("{pad}Show project metadata and managed paths");
    outln!("  {cmd} {USAGE_PERMISSION_GRANT}");
    outln!("{pad}Grant temporary permission for a local path");
    outln!("  {cmd} {USAGE_PERMISSION_VIEW}");
    outln!("{pad}Show active default and temporary permissions");
    outln!("  {cmd} {USAGE_PERMISSION_REVOKE}");
    outln!("{pad}Revoke a temporary permission");
    outln!("  {cmd} {USAGE_SEARCH}");
    outln!("{pad}Search accessible workspace knowledge, ranked by relevance");
    outln!("  {cmd} --version Show the {cmd} binary version");
    outln!("  {cmd} --help    Show this help message");
    outln!();
    outln!("Environment:");
    outln!(
        "  AI_WORKSPACE_HOME  Override the workspace folder for this shell (wins over the saved setting)"
    );
}

fn print_version() -> Result<()> {
    outln!("{COMMAND_NAME} {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
