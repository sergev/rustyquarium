mod animation;
mod cli;
mod depth;
mod entity;
mod environment;
mod fish;
mod info;
mod special;

/// This is the program entry point.
/// It sends command-line arguments to `run_cli`.
/// If something fails, it prints an error and exits.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = cli::run_cli(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
