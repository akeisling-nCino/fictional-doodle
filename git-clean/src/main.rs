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
        eprintln!("git remote prune failed");
        std::process::exit(1);
    }

    let output = std::process::Command::new("git")
        .args(["branch", "-vv"])
        .output()
        .expect("Failed to run git branch");

    let stdout = String::from_utf8(output.stdout).expect("git output was not valid UTF-8");

    let gone_branches: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains(": gone"))
        .map(|line| {
            line.trim()
                .trim_start_matches('*')
                .trim()
                .split_whitespace()
                .next()
                .unwrap()
        })
        .collect();

    println!("Found {} branches to delete", gone_branches.len());
    for branch in &gone_branches {
        println!("{branch}")
    }
}
