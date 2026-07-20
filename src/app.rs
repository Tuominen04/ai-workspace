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
    Help,
}

fn parse_command(args: Vec<String>) -> Result<Command> {
    if args.is_empty() {
        return Ok(Command::Run);
    }

    match args[0].as_str() {
        "init" => Ok(Command::Init),
        "status" => Ok(Command::Status),
        "-h" | "--help" | "help" => Ok(Command::Help),
        other => Err(AppError::Message(format!(
            "unknown command '{other}'. Use 'ai --help' for usage."
        ))),
    }
}

fn print_help() {
    println!("AI Workspace");
    println!();
    println!("Usage:");
    println!("  ai           Launch an interactive workspace session");
    println!("  ai init      Create the standard ~/AI directory layout");
    println!("  ai status    Show discovered workspaces and root paths");
    println!("  ai --help    Show this help message");
    println!();
    println!("Environment:");
    println!("  AI_WORKSPACE_HOME  Override the default workspace root (~/AI)");
}
