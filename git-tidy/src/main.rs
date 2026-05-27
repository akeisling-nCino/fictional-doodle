use clap::Parser;
use colored::Colorize;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::thread;

#[derive(Parser)]
#[command(about = "Delete local branches whose remote has been merged and deleted")]
struct Cli {
    #[arg(short, long, help = "Directory to scan for git repos")]
    dir: Option<PathBuf>,
}

fn clean_repo(path: &Path) -> String {
    let mut output = String::new();
    writeln!(output, "{}", format!("Repo: {}", path.display()).bold()).unwrap();
    writeln!(
        output,
        "{}",
        "  Pruning stale remote tracking branches...".cyan().bold()
    )
    .unwrap();

    let prune_args = vec!["remote", "prune", "origin"];

    let prune = Command::new("git")
        .args(&prune_args)
        .current_dir(path)
        .output()
        .expect("Failed to run git remote prune");

    if !prune.status.success() {
        writeln!(output, "{}", "git remote prune failed".red().bold()).unwrap();
    }

    let prune_output = String::from_utf8(prune.stdout).expect("git output was not valid UTF-8");

    for line in prune_output.lines() {
        if line.contains("[pruned]") {
            let branch = line
                .trim()
                .trim_start_matches("*")
                .trim()
                .replace("[would prune]", "")
                .replace("[pruned]", "")
                .trim()
                .to_string();
            writeln!(output, "    {} {}", "Pruned:".yellow(), branch).unwrap();
        }
    }

    let branch_output = Command::new("git")
        .args(["branch", "-vv"])
        .current_dir(path)
        .output()
        .expect("Failed to run git branch");

    let stdout = String::from_utf8(branch_output.stdout).expect("git output was not valid UTF-8");

    let gone_branches: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains(": gone"))
        .map(|line| {
            line.trim()
                .trim_start_matches('*')
                .split_whitespace()
                .next()
                .unwrap()
        })
        .collect();

    if gone_branches.is_empty() {
        writeln!(output, "{}", "  No local branches to delete.".green()).unwrap();
        return output;
    }

    let protected = ["main", "master", "develop", "release"];

    for branch in &gone_branches {
        if protected.contains(branch) {
            writeln!(
                output,
                "{}",
                format!("  Skipping protected branch: {branch}")
                    .yellow()
                    .bold()
            )
            .unwrap();
            continue;
        }
        let result = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(path)
            .output()
            .expect("Failed to run git branch -D");

        if result.status.success() {
            writeln!(output, "{}", format!("  Deleted: {branch}").red()).unwrap();
        } else {
            writeln!(output, "{}", format!("  Failed to delete: {branch}").red().bold()).unwrap();
        }
    }
    output
}

fn main() {
    let cli = Cli::parse();
    let current_dir = match cli.dir {
        Some(path) => path,
        None => env::current_dir().expect("Could not get current directory"),
    };

    if current_dir.join(".git").exists() {
        print!("{}", clean_repo(&current_dir));
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

    let handles: Vec<_> = repos
        .into_iter()
        .map(|repo| thread::spawn(move || clean_repo(&repo)))
        .collect();

    for handle in handles {
        print!("{}", handle.join().unwrap());
    }
}
