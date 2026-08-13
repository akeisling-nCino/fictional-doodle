use clap::Parser;
use colored::Colorize;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(about = "Authenticate with AWS SSO and launch opencode")]
struct Cli {
    #[arg(short, long, help = "Directory to launch opencode from")]
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

fn launch_opencode(directory: Option<PathBuf>) {
    let mut cmd = Command::new("opencode");
    cmd.env("AWS_PROFILE", "genai");

    if let Some(dir) = directory {
        cmd.current_dir(dir);
    }
    let err = cmd.exec();

    eprintln!("{} {}", "Failed to launch opencode:".red(), err);
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    run_sso_login();
    launch_opencode(cli.dir);
}
