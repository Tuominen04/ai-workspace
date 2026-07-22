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
        Command::Validate => session::validate_workspace_state(&paths),
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
    ProjectCreate { name: String, description: Option<String> },
    ProjectShow { project: String },
    Validate,
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
        "-h" | "--help" | "help" => Ok(Command::Help),
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
            "missing project subcommand. Use 'ai project create' or 'ai project show'."
                .to_string(),
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
            "invalid project command. Use 'ai project create' or 'ai project show'."
                .to_string(),
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

fn print_help() {
    println!("AI Workspace");
    println!();
    println!("Usage:");
    println!("  ai           Launch an interactive workspace session");
    println!("  ai init      Create the standard ~/AI directory layout");
    println!("  ai status    Show discovered workspaces and root paths");
    println!("  ai validate  Validate workspace and project definitions");
    println!("  ai config show");
    println!("              Show workspace configuration");
    println!("  ai project create --name NAME [--description TEXT]");
    println!("              Create a managed project workspace");
    println!("  ai project show PROJECT");
    println!("              Show project metadata and managed paths");
    println!("  ai --help    Show this help message");
    println!();
    println!("Environment:");
    println!("  AI_WORKSPACE_HOME  Override the default workspace root (~/AI)");
}
