use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
#[command(about = "Delete local branches whose remote has been merged and deleted")]
struct Cli {
    #[arg(short, long, help = "Preview branches to delete without deleting them")]
    preview: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("Pruning stale remote tracking branches...");

    let prune = std::process::Command::new("git")
        .args(["remote", "prune", "origin"])
        .status()
        .expect("Failed to run git");

    if !prune.success() {
        eprintln!("git rmeote prune failed");
        std::process::exit(1);
    }
}
