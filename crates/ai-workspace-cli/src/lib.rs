/// Like `println!`, but a closed stdout (for example `ai-workspace --help | head`)
/// does not panic. See [`ui::write_stdout`].
macro_rules! outln {
    () => {
        $crate::ui::write_stdout(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::ui::write_stdout(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// Like `print!`, but a closed stdout does not panic. See [`ui::write_stdout`].
macro_rules! out {
    ($($arg:tt)*) => {
        $crate::ui::write_stdout(format_args!($($arg)*))
    };
}

pub mod app;
pub mod config;
pub mod context;
pub mod error;
pub mod paths;
pub mod runtime;
pub mod search;
pub mod session;
pub mod ui;
pub mod workspace;

/// The name of the installed command.
///
/// This is the single source for the command's name in every user-facing
/// message (help text, usage errors, hints, the version line). Build such
/// strings from this constant instead of writing the name out.
pub const COMMAND_NAME: &str = "ai-workspace";
