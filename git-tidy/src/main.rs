use clap::Parser;
use colored::Colorize;
use std::env;
use std::fmt::Write;
use std::fs;
use std::string::FromUtf8Error;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::thread;

#[derive(Debug)]
enum TidyError {
    Io(std::io::Error),
    Utf8(FromUtf8Error)
}

impl std::fmt::Display for TidyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TidyError::Io(e) => write!(f, "IO error: {e}"),
            TidyError::Utf8(e) => write!(f, "UTF-8 error: {e}"),
        }
    }
}

impl std::error::Error for TidyError {}

impl From<std::io::Error> for TidyError {
    fn from(e: std::io::Error) -> Self {
        TidyError::Io(e)
    }
}
impl From<FromUtf8Error> for TidyError {
    fn from(e: FromUtf8Error) -> Self {
        TidyError::Utf8(e)
    }
}

#[derive(Parser)]
#[command(about = "Delete local branches whose remote has been merged and deleted")]
struct Cli {
    #[arg(short, long, help = "Directory to scan for git repos")]
    dir: Option<PathBuf>,
}

fn clean_repo(path: &Path) -> Result<String, TidyError> {
    let mut output = String::new();
    writeln!(output, "{}", format!("Repo: {}", path.display()).bold());
    writeln!(
        output,
        "{}",
        "  Pruning stale remote tracking branches...".cyan().bold()
    );

    let prune_args = vec!["remote", "prune", "origin"];

    let prune = Command::new("git")
        .args(&prune_args)
        .current_dir(path)
        .output()?;

    if !prune.status.success() {
        writeln!(output, "    {}", "git remote prune failed".red().bold());
    }

    let prune_output = String::from_utf8(prune.stdout)?;

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
            writeln!(output, "    {} {}", "Pruned:".yellow(), branch);
        }
    }

    let branch_output = Command::new("git")
        .args(["branch", "-vv"])
        .current_dir(path)
        .output()?;

    let stdout = String::from_utf8(branch_output.stdout)?;

    let gone_branches: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains(": gone"))
        .filter_map(|line| {
            line.trim()
                .trim_start_matches('*')
                .split_whitespace()
                .next()
        })
        .collect();
    if gone_branches.is_empty() {
        writeln!(output, "{}", "  No local branches to delete.".green());
        return Ok(output);
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
            );
            continue;
        }
        let result = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(path)
            .output()?;

        if result.status.success() {
            writeln!(output, "{}", format!("  Deleted: {branch}").red());
        } else {
            writeln!(output, "{}", format!("  Failed to delete: {branch}").red().bold());
        }
    }
    Ok(output)
}

fn main() -> Result<(), TidyError> {
    let cli = Cli::parse();
    let current_dir = match cli.dir {
        Some(path) => path,
        None => env::current_dir()?,
    };

    if current_dir.join(".git").exists() {
        print!("{}", clean_repo(&current_dir)?);
        return Ok(());
    }

    let repos: Vec<PathBuf> = fs::read_dir(&current_dir)?
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
        match handle.join() {
            Ok(Ok(out)) => println!("{out}"),
            Ok(Err(e)) => eprintln!("Error: {e}"),
            Err(_) => eprintln!("A worker thread panicked")
        }
    }
    Ok(())
}
