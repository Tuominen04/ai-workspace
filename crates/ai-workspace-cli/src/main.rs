fn main() {
    if let Err(error) = ai_workspace::app::run() {
        ai_workspace::app::report_error(&error);
        std::process::exit(1);
    }
}
