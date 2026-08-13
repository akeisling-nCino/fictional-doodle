use clap::Parser;
use colored::Colorize;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(about = "Authenticate with AWS SSO and launch Codex")]
struct Cli {
    #[arg(short, long, help = "Directory to launch Codex from")]
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

fn resolve_github_pat_token(
    github_pat_token: Option<OsString>,
    github_token: Option<OsString>,
) -> Option<OsString> {
    github_pat_token.or(github_token)
}

fn launch_codex(directory: Option<PathBuf>) {
    let github_pat_token = resolve_github_pat_token(
        std::env::var_os("GITHUB_PAT_TOKEN"),
        std::env::var_os("GITHUB_TOKEN"),
    );

    let Some(github_pat_token) = github_pat_token else {
        eprintln!(
            "{}",
            "GITHUB_PAT_TOKEN is not set. Set GITHUB_PAT_TOKEN or GITHUB_TOKEN before starting Codex."
                .red()
        );
        std::process::exit(1);
    };

    let mut command = Command::new("codex");
    command
        .env("AWS_PROFILE", "genai")
        .env("GITHUB_PAT_TOKEN", github_pat_token);

    if let Some(dir) = directory {
        command.current_dir(dir);
    }

    let error = command.exec();
    eprintln!("{} {}", "Failed to launch Codex:".red(), error);
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    run_sso_login();
    launch_codex(cli.dir);
}

#[cfg(test)]
mod tests {
    use super::{Cli, resolve_github_pat_token};
    use clap::Parser;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn accepts_a_directory_argument() {
        let cli = Cli::try_parse_from(["start-codex", "--dir", "/tmp/project"])
            .expect("directory argument should parse");

        assert_eq!(cli.dir, Some(PathBuf::from("/tmp/project")));
    }

    #[test]
    fn uses_the_current_directory_when_no_directory_argument_is_provided() {
        let cli = Cli::try_parse_from(["start-codex"]).expect("no directory argument should parse");

        assert_eq!(cli.dir, None);
    }

    #[test]
    fn uses_github_pat_token_when_available() {
        let token = resolve_github_pat_token(
            Some(OsString::from("preferred-token")),
            Some(OsString::from("fallback-token")),
        );

        assert_eq!(token, Some(OsString::from("preferred-token")));
    }

    #[test]
    fn uses_github_token_when_pat_token_is_unavailable() {
        let token = resolve_github_pat_token(None, Some(OsString::from("fallback-token")));

        assert_eq!(token, Some(OsString::from("fallback-token")));
    }

    #[test]
    fn returns_none_when_no_github_token_is_available() {
        let token = resolve_github_pat_token(None, None);

        assert_eq!(token, None);
    }
}
