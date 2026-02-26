use clap::Parser;
use colored::Colorize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Parser)]
#[command(about = "Delete local branches whose remote has been merged and deleted")]
struct Cli {
    #[arg(short, long, help = "Preview branches to delete without deleting them")]
    preview: bool,

    #[arg(short, long, help = "Directory to scan for git repos")]
    dir: Option<PathBuf>,
}

fn clean_repo(path: &Path, preview: bool) {
    println!("\n{}", format!("Repo: {}", path.display()).bold());
    println!(
        "{}",
        "  Pruning stale remote tracking branches...".cyan().bold()
    );

    let mut prune_args = vec!["remote", "prune", "origin"];
    if preview {
        prune_args.push("--dry-run");
    }

    let prune = Command::new("git")
        .args(&prune_args)
        .current_dir(path)
        .output()
        .expect("Failed to run git remote prune");

    if !prune.status.success() {
        eprintln!("{}", "git remote prune failed".red().bold());
        return;
    }

    let prune_output = String::from_utf8(prune.stdout).expect("git output was not valid UTF-8");

    for line in prune_output.lines() {
        if line.contains("[would prune]") || line.contains("[pruned]") {
            let branch = line
                .trim()
                .trim_start_matches("*")
                .trim()
                .replace("[would prune]", "")
                .replace("[pruned]", "")
                .trim()
                .to_string();
            if preview {
                println!("    {} {}", "Would prune:".yellow(), branch);
            } else {
                println!("    {} {}", "Pruned:".yellow(), branch);
            }
        }
    }

    let output = Command::new("git")
        .args(["branch", "-vv"])
        .current_dir(path)
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

    if gone_branches.is_empty() {
        if !preview {
            println!("{}", "  No local branches to delete.".green());
        }
        return;
    }

    let protected = ["main", "master", "develop", "release"];

    for branch in &gone_branches {
        if protected.contains(branch) {
            println!(
                "{}",
                format!("  Skipping protected branch: {branch}")
                    .yellow()
                    .bold()
            );
            continue;
        }

        if preview {
            println!("{}", format!("  Would delete: {branch}").yellow());
        } else {
            let result = Command::new("git")
                .args(["branch", "-D", branch])
                .current_dir(path)
                .output()
                .expect("Failed to run git branch -D");

            if result.status.success() {
                println!("{}", format!("  Deleted: {branch}").red());
            } else {
                eprintln!("{}", format!("  Failed to delete: {branch}").red().bold());
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let current_dir = match cli.dir {
        Some(path) => path,
        None => env::current_dir().expect("Could not get current directory"),
    };

    if current_dir.join(".git").exists() {
        clean_repo(&current_dir, cli.preview);
        return;
    }

    let repos: Vec<PathBuf> = fs::read_dir(&current_dir)
        .expect("Could not read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join(".git").exists())
        .collect();

    if repos.is_empty() {
        println!("{}", "No git repos found.".yellow());
        process::exit(1);
    }

    for repo in &repos {
        clean_repo(repo, cli.preview);
    }
}
