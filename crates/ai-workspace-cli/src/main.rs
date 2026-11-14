fn main() {
    if let Err(error) = ai_workspace::app::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
