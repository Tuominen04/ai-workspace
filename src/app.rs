use crate::error::{AppError, Result};
use crate::paths::WorkspacePaths;
use crate::session;

pub fn run() -> Result<()> {
    let command = parse_command(std::env::args().skip(1).collect())?;
    let paths = WorkspacePaths::detect()?;

    match command {
        Command::Run => session::run_session(&paths),
        Command::Init => session::init_workspace(&paths),
        Command::Status => session::print_status(&paths),
        Command::ConfigShow => session::show_workspace_config(&paths),
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
        Command::Version => print_version(),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

enum Command {
    Run,
    Init,
    Status,
    ConfigShow,
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
            "unknown command '{other}'. Use 'ai --help' for usage."
        ))),
    }
}

fn parse_config_command(args: &[String]) -> Result<Command> {
    if args.len() == 1 && args[0] == "show" {
        return Ok(Command::ConfigShow);
    }

    Err(AppError::Message(
        "invalid config command. Use 'ai config show'.".to_string(),
    ))
}

fn parse_project_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        return Err(AppError::Message(
            "missing project subcommand. Use 'ai project create' or 'ai project show'.".to_string(),
        ));
    }

    match args[0].as_str() {
        "show" => {
            if args.len() != 2 {
                return Err(AppError::Message(
                    "usage: ai project show PROJECT".to_string(),
                ));
            }
            Ok(Command::ProjectShow {
                project: args[1].clone(),
            })
        }
        "create" => parse_project_create_command(&args[1..]),
        _ => Err(AppError::Message(
            "invalid project command. Use 'ai project create' or 'ai project show'.".to_string(),
        )),
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
                    return Err(AppError::Message(
                        "usage: ai project create --name NAME [--description TEXT]".to_string(),
                    ));
                }
                name = Some(args[index + 1].clone());
                index += 2;
            }
            "--description" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai project create --name NAME [--description TEXT]".to_string(),
                    ));
                }
                description = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(AppError::Message(
                    "usage: ai project create --name NAME [--description TEXT]".to_string(),
                ));
            }
        }
    }

    let name = name.ok_or_else(|| {
        AppError::Message("usage: ai project create --name NAME [--description TEXT]".to_string())
    })?;

    Ok(Command::ProjectCreate { name, description })
}

fn parse_permission_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        return Err(AppError::Message(
            "missing permission subcommand. Use 'ai permission grant', 'ai permission view', or 'ai permission revoke'."
                .to_string(),
        ));
    }

    match args[0].as_str() {
        "grant" => parse_permission_grant_command(&args[1..]),
        "view" => parse_permission_view_command(&args[1..]),
        "revoke" => parse_permission_revoke_command(&args[1..]),
        _ => Err(AppError::Message(
            "invalid permission command. Use 'ai permission grant', 'ai permission view', or 'ai permission revoke'."
                .to_string(),
        )),
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
                    return Err(AppError::Message(
                        "usage: ai permission grant [--project PROJECT] --path PATH".to_string(),
                    ));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--path" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai permission grant [--project PROJECT] --path PATH".to_string(),
                    ));
                }
                path = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(AppError::Message(
                    "usage: ai permission grant [--project PROJECT] --path PATH".to_string(),
                ));
            }
        }
    }

    let path = path.ok_or_else(|| {
        AppError::Message("usage: ai permission grant [--project PROJECT] --path PATH".to_string())
    })?;

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
                    return Err(AppError::Message(
                        "usage: ai permission revoke [--project PROJECT] --path PATH".to_string(),
                    ));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--path" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai permission revoke [--project PROJECT] --path PATH".to_string(),
                    ));
                }
                path = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                return Err(AppError::Message(
                    "usage: ai permission revoke [--project PROJECT] --path PATH".to_string(),
                ));
            }
        }
    }

    let path = path.ok_or_else(|| {
        AppError::Message("usage: ai permission revoke [--project PROJECT] --path PATH".to_string())
    })?;

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
                    return Err(AppError::Message(
                        "usage: ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]"
                            .to_string(),
                    ));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--filter" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]"
                            .to_string(),
                    ));
                }
                filter = Some(args[index + 1].clone());
                index += 2;
            }
            "--page" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]"
                            .to_string(),
                    ));
                }
                page = args[index + 1].parse::<usize>().map_err(|_| {
                    AppError::Message("--page must be a positive integer".to_string())
                })?;
                index += 2;
            }
            "--page-size" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]"
                            .to_string(),
                    ));
                }
                page_size = args[index + 1].parse::<usize>().map_err(|_| {
                    AppError::Message("--page-size must be a positive integer".to_string())
                })?;
                index += 2;
            }
            _ => {
                return Err(AppError::Message(
                    "usage: ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]"
                        .to_string(),
                ));
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
                    return Err(AppError::EmptySearchQuery);
                }
                query = Some(args[index + 1].clone());
                index += 2;
            }
            "--project" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai search --query TEXT [--project PROJECT] [--limit N]".to_string(),
                    ));
                }
                project = Some(args[index + 1].clone());
                index += 2;
            }
            "--limit" => {
                if index + 1 >= args.len() {
                    return Err(AppError::Message(
                        "usage: ai search --query TEXT [--project PROJECT] [--limit N]".to_string(),
                    ));
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
                return Err(AppError::Message(
                    "usage: ai search --query TEXT [--project PROJECT] [--limit N]".to_string(),
                ));
            }
        }
    }

    let query = match query {
        Some(query) if !query.trim().is_empty() => query,
        _ => return Err(AppError::EmptySearchQuery),
    };

    Ok(Command::Search {
        query,
        project,
        limit,
    })
}

fn print_help() {
    println!("AI Workspace");
    println!();
    println!("Usage:");
    println!("  ai           Launch an interactive workspace session");
    println!("  ai init      Create the standard ~/AI directory layout");
    println!("  ai status    Show discovered workspaces and root paths");
    println!("  ai validate  Validate workspace and project definitions");
    println!("  ai config show");
    println!("Show workspace configuration");
    println!("  ai project create --name NAME [--description TEXT]");
    println!("              Create a managed project workspace");
    println!("  ai project show PROJECT");
    println!("              Show project metadata and managed paths");
    println!("  ai permission grant [--project PROJECT] --path PATH");
    println!("              Grant temporary permission for a local path");
    println!("  ai permission view [--project PROJECT] [--filter TEXT] [--page N] [--page-size N]");
    println!("              Show active default and temporary permissions");
    println!("  ai permission revoke [--project PROJECT] --path PATH");
    println!("              Revoke a temporary permission");
    println!("  ai search --query TEXT [--project PROJECT] [--limit N]");
    println!("              Search accessible workspace knowledge, ranked by relevance");
    println!("  ai --version Show the ai binary version");
    println!("  ai --help    Show this help message");
    println!();
    println!("Environment:");
    println!("  AI_WORKSPACE_HOME  Override the default workspace root (~/AI)");
}

fn print_version() -> Result<()> {
    println!("ai {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
