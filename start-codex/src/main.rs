use clap::Parser;
use colored::Colorize;
use serde_json::Value;
use std::ffi::OsString;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{Array, DocumentMut, Item, Table, value};

const CATALOG_WINDOW: u64 = 1_050_000;
const CODEX_MODEL: &str = "gpt-5.6-terra";
const GATEWAY_BASE_URL: &str = "https://gptgateway.ncino.ai/openai/v1";
const GATEWAY_AUTH_COMMAND: &str = "tok=$(aws eks get-token --cluster-name gpt-gateway --region us-east-1 --profile genai --output text --query status.token 2>/dev/null) && printf '%s' \"$tok\" || { echo 'GptGateway: could not mint a token. Check the genai profile exists, then run: aws sso login --profile genai' >&2; exit 1; }";
const GENAI_ACCOUNT: &str = "714322698969";

#[derive(Parser)]
#[command(about = "Authenticate with AWS SSO and launch Codex")]
struct Cli {
    #[arg(short, long, help = "Directory to launch Codex from")]
    dir: Option<PathBuf>,
}

fn run_sso_login() {
    let status = Command::new("aws")
        .args([
            "sso",
            "login",
            "--profile",
            "genai",
            "--region",
            "us-east-1",
        ])
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

fn is_genai_account(account: &str) -> bool {
    account.trim() == GENAI_ACCOUNT
}

fn verify_genai_account() {
    let output = Command::new("aws")
        .args([
            "sts",
            "get-caller-identity",
            "--profile",
            "genai",
            "--region",
            "us-east-1",
            "--query",
            "Account",
            "--output",
            "text",
        ])
        .output()
        .expect("Failed to run aws");

    if output.status.success() && is_genai_account(&String::from_utf8_lossy(&output.stdout)) {
        return;
    }

    eprintln!(
        "{}",
        format!("The genai profile must resolve to R&D GenAI account {GENAI_ACCOUNT}.").red()
    );
    std::process::exit(1);
}

fn codex_home() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }

    let home = std::env::var_os("HOME").ok_or("HOME is not set and CODEX_HOME was not provided")?;
    Ok(PathBuf::from(home).join(".codex"))
}

fn configure_gateway(existing: &str, catalog_path: &str) -> Result<String, String> {
    let mut config = existing
        .parse::<DocumentMut>()
        .map_err(|error| format!("Could not parse ~/.codex/config.toml: {error}"))?;

    config["model_provider"] = value("gpt-gateway");
    config["model"] = value(CODEX_MODEL);
    config["model_catalog_json"] = value(catalog_path);

    if config.get("model_providers").is_none() {
        config["model_providers"] = Item::Table(Table::new());
    }
    let providers = config["model_providers"]
        .as_table_mut()
        .ok_or("model_providers in ~/.codex/config.toml must be a table")?;
    if !providers.contains_key("gpt-gateway") {
        providers["gpt-gateway"] = Item::Table(Table::new());
    }
    let gateway = providers["gpt-gateway"]
        .as_table_mut()
        .ok_or("model_providers.gpt-gateway in ~/.codex/config.toml must be a table")?;

    gateway["name"] = value("GPT via nCino GptGateway");
    gateway["base_url"] = value(GATEWAY_BASE_URL);
    gateway["wire_api"] = value("responses");
    gateway["requires_openai_auth"] = value(false);

    if !gateway.contains_key("auth") {
        gateway["auth"] = Item::Table(Table::new());
    }
    let auth = gateway["auth"]
        .as_table_mut()
        .ok_or("model_providers.gpt-gateway.auth in ~/.codex/config.toml must be a table")?;
    let mut args = Array::new();
    args.push("-c");
    args.push(GATEWAY_AUTH_COMMAND);
    auth["command"] = value("bash");
    auth["args"] = value(args);

    Ok(config.to_string())
}

fn find_catalog_start(binary: &[u8]) -> Option<usize> {
    [b"{\n  \"models\"".as_slice(), b"{\"models\"".as_slice()]
        .iter()
        .find_map(|marker| {
            binary
                .windows(marker.len())
                .position(|window| window == *marker)
        })
}

fn extract_catalog(binary: &[u8]) -> Result<Value, String> {
    let start = find_catalog_start(binary)
        .ok_or("Could not find the embedded model catalog in the Codex binary")?;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, byte) in binary[start..].iter().copied().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'\"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_slice(&binary[start..=start + offset]).map_err(
                        |error| format!("Could not parse the embedded model catalog: {error}"),
                    );
                }
            }
            _ => {}
        }
    }

    Err("The embedded model catalog was not terminated".to_owned())
}

fn reset_context_window(catalog: &mut Value) -> Result<(), String> {
    let models = catalog["models"]
        .as_array_mut()
        .ok_or("The embedded model catalog has no models array")?;
    let mut updated = 0;

    for model in models {
        if model["slug"]
            .as_str()
            .is_some_and(|slug| slug.starts_with("gpt-5.6"))
        {
            model["context_window"] = Value::from(CATALOG_WINDOW);
            model["max_context_window"] = Value::from(CATALOG_WINDOW);
            model["effective_context_window_percent"] = Value::from(100);
            updated += 1;
        }
    }

    if updated == 0 {
        return Err("The embedded model catalog has no GPT-5.6 models".to_owned());
    }

    Ok(())
}

// Multi-agent v2 currently omits collaboration tools for the gateway's Sol and Terra models.
// Keep named custom-agent delegation on v1 until the v2 runtime exposes those tools.
fn use_multi_agent_v1(catalog: &mut Value) -> Result<(), String> {
    let models = catalog["models"]
        .as_array_mut()
        .ok_or("The embedded model catalog has no models array")?;
    let mut updated = 0;

    for model in models {
        if model["slug"]
            .as_str()
            .is_some_and(|slug| slug.starts_with("gpt-5.6"))
        {
            model["multi_agent_version"] = Value::from("v1");
            updated += 1;
        }
    }

    if updated == 0 {
        return Err("The embedded model catalog has no GPT-5.6 models".to_owned());
    }

    Ok(())
}

fn resolve_codex_command() -> Result<PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or("PATH is not set")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("codex");
        if candidate.is_file() {
            return candidate
                .canonicalize()
                .map_err(|error| format!("Could not resolve {}: {error}", candidate.display()));
        }
    }

    Err("Could not find codex on PATH".to_owned())
}

fn find_native_codex_binary() -> Result<PathBuf, String> {
    let command = resolve_codex_command()?;
    if let Ok(binary) = fs::read(&command)
        && extract_catalog(&binary).is_ok()
    {
        return Ok(command);
    }

    let package_root = command
        .parent()
        .and_then(Path::parent)
        .ok_or("Could not determine the Codex package directory")?;
    let node_modules = package_root.join("node_modules");
    let platform_package = match std::env::consts::OS {
        "macos" if std::env::consts::ARCH == "aarch64" => "@openai/codex-darwin-arm64",
        "macos" if std::env::consts::ARCH == "x86_64" => "@openai/codex-darwin-x64",
        "linux" if std::env::consts::ARCH == "aarch64" => "@openai/codex-linux-arm64",
        "linux" if std::env::consts::ARCH == "x86_64" => "@openai/codex-linux-x64",
        operating_system => {
            return Err(format!(
                "Unsupported operating system or architecture: {operating_system}/{}",
                std::env::consts::ARCH
            ));
        }
    };
    let vendor_directory = node_modules.join(platform_package).join("vendor");
    let entries = fs::read_dir(&vendor_directory)
        .map_err(|error| format!("Could not read {}: {error}", vendor_directory.display()))?;

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Could not inspect Codex vendor directory: {error}"))?;
        let candidate = entry.path().join("bin").join("codex");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err("Could not find the native Codex binary".to_owned())
}

fn write_configuration() -> Result<(), String> {
    let codex_home = codex_home()?;
    fs::create_dir_all(&codex_home)
        .map_err(|error| format!("Could not create {}: {error}", codex_home.display()))?;

    let native_binary = find_native_codex_binary()?;
    let mut catalog = extract_catalog(
        &fs::read(&native_binary)
            .map_err(|error| format!("Could not read {}: {error}", native_binary.display()))?,
    )?;
    reset_context_window(&mut catalog)?;
    use_multi_agent_v1(&mut catalog)?;

    let catalog_path = codex_home.join("model-catalog.json");
    let catalog_contents = serde_json::to_string_pretty(&catalog)
        .map_err(|error| format!("Could not serialize the model catalog: {error}"))?;
    fs::write(&catalog_path, format!("{catalog_contents}\n"))
        .map_err(|error| format!("Could not write {}: {error}", catalog_path.display()))?;

    let config_path = codex_home.join("config.toml");
    let existing = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Could not read {}: {error}", config_path.display())),
    };
    let configured = configure_gateway(&existing, &catalog_path.to_string_lossy())?;

    if configured != existing {
        if config_path.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    format!("Could not create a configuration backup timestamp: {error}")
                })?
                .as_secs();
            let backup_path = codex_home.join(format!("config.toml.bak.{timestamp}"));
            fs::copy(&config_path, &backup_path).map_err(|error| {
                format!(
                    "Could not back up {} to {}: {error}",
                    config_path.display(),
                    backup_path.display()
                )
            })?;
        }
        fs::write(&config_path, configured)
            .map_err(|error| format!("Could not write {}: {error}", config_path.display()))?;
    }

    Ok(())
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
    verify_genai_account();
    if let Err(error) = write_configuration() {
        eprintln!("{} {error}", "Codex configuration failed:".red());
        std::process::exit(1);
    }
    launch_codex(cli.dir);
}

#[cfg(test)]
mod tests {
    use super::{
        CATALOG_WINDOW, Cli, configure_gateway, extract_catalog, is_genai_account,
        reset_context_window, resolve_github_pat_token, use_multi_agent_v1,
    };
    use clap::Parser;
    use serde_json::json;
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

    #[test]
    fn configures_the_gateway_and_catalog_path() {
        let config = configure_gateway("existing = \"value\"\n", "/tmp/model-catalog.json")
            .expect("gateway configuration should be valid TOML");

        assert!(config.contains("model_provider = \"gpt-gateway\""));
        assert!(config.contains("model = \"gpt-5.6-terra\""));
        assert!(config.contains("base_url = \"https://gptgateway.ncino.ai/openai/v1\""));
        assert!(config.contains("model_catalog_json = \"/tmp/model-catalog.json\""));
        assert!(config.contains("--cluster-name gpt-gateway"));
        assert!(config.contains("--profile genai"));
    }

    #[test]
    fn resets_all_gpt_56_context_windows() {
        let mut catalog = json!({
            "models": [
                {"slug": "gpt-5.6-terra", "context_window": 272000, "max_context_window": 272000, "effective_context_window_percent": 95},
                {"slug": "gpt-5.6-sol", "context_window": 100, "max_context_window": 100, "effective_context_window_percent": 1},
                {"slug": "gpt-4.1", "context_window": 100, "max_context_window": 100, "effective_context_window_percent": 95}
            ]
        });

        reset_context_window(&mut catalog).expect("catalog should contain GPT-5.6 models");

        assert_eq!(catalog["models"][0]["context_window"], CATALOG_WINDOW);
        assert_eq!(catalog["models"][0]["max_context_window"], CATALOG_WINDOW);
        assert_eq!(
            catalog["models"][0]["effective_context_window_percent"],
            100
        );
        assert_eq!(catalog["models"][1]["context_window"], CATALOG_WINDOW);
        assert_eq!(catalog["models"][2]["context_window"], 100);
    }

    #[test]
    fn uses_multi_agent_v1_for_all_gpt_56_models() {
        let mut catalog = json!({
            "models": [
                {"slug": "gpt-5.6-terra", "multi_agent_version": "v2"},
                {"slug": "gpt-5.6-sol", "multi_agent_version": "v2"},
                {"slug": "gpt-5.6-luna", "multi_agent_version": "v1"},
                {"slug": "gpt-5.5", "multi_agent_version": null}
            ]
        });

        use_multi_agent_v1(&mut catalog).expect("catalog should contain GPT-5.6 models");

        assert_eq!(catalog["models"][0]["multi_agent_version"], "v1");
        assert_eq!(catalog["models"][1]["multi_agent_version"], "v1");
        assert_eq!(catalog["models"][2]["multi_agent_version"], "v1");
        assert_eq!(catalog["models"][3]["multi_agent_version"], json!(null));
    }

    #[test]
    fn extracts_the_embedded_catalog_from_a_native_binary() {
        let binary = b"prefix {\n  \"models\": [{\"slug\": \"gpt-5.6-terra\"}]\n} suffix";

        let catalog = extract_catalog(binary).expect("catalog should be extracted");

        assert_eq!(catalog["models"][0]["slug"], "gpt-5.6-terra");
    }

    #[test]
    fn recognizes_the_required_genai_account() {
        assert!(is_genai_account("714322698969\n"));
        assert!(!is_genai_account("123456789012"));
    }
}
