use clap::Parser;
use colored::Colorize;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(about = "Authenticate with AWS SSO and launch claude")]
struct Cli {
    #[arg(short, long, help = "Directory to launch claude from")]
    dir: Option<PathBuf>,
}

fn run_sso_login() {
    let status = Command::new("aws")
        .args(["sso", "login", "--profile", "genai"])
        .status()
        .expect("Failed to run aws");

    if status.success() {
        println!("{}", "AWS SSO Login successful!".green());
    } else {
        eprintln!(
            "{}",
            "AWS SSO Login failed. Check your credentials and try again".red()
        );
        std::process::exit(1);
    }
}

fn launch_claude(directory: Option<PathBuf>) {
    let mut cmd = Command::new("claude");
    cmd.env("AWS_PROFILE", "genai");

    if let Some(dir) = directory {
        cmd.current_dir(dir);
    }
    let err = cmd.exec();

    eprintln!("{} {}", "Failed to launch claude:".red(), err);
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    run_sso_login();
    launch_claude(cli.dir);
}
