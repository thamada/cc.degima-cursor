//! 接続先に通る SSH 秘密鍵の解決。

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ssh_cmd::SshTarget;
use crate::ssh_config::expand_home;

/// 明示指定が無いときの候補（先勝ち。実接続で認証できるものを採用）。
fn default_candidate_keys() -> Vec<PathBuf> {
    let mut keys = Vec::new();

    // ターミナル既定に近い鍵を優先。
    keys.push(expand_home("~/.ssh/id_ed25519"));
    keys.push(expand_home("~/.ssh/id_rsa"));
    keys.push(expand_home("~/.ssh/cursor2_id_ed25519"));

    // degima-agent が Dashboard 用に生成する鍵。
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            keys.push(
                PathBuf::from(home)
                    .join("Library/Application Support/degima-agent/ssh-keys/degima_dashboard_ed25519"),
            );
        }
    }
    #[cfg(target_os = "linux")]
    {
        keys.push(expand_home(
            "~/.config/degima-agent/ssh-keys/degima_dashboard_ed25519",
        ));
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            keys.push(
                PathBuf::from(appdata)
                    .join("degima-agent")
                    .join("ssh-keys")
                    .join("degima_dashboard_ed25519"),
            );
        }
    }

    keys
}

fn key_authenticates(target: &SshTarget, identity_path: &Path) -> bool {
    let status = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "IdentitiesOnly=yes",
            "-i",
        ])
        .arg(identity_path)
        .arg("-p")
        .arg(target.port.to_string())
        .arg(format!("{}@{}", target.user, target.host))
        .arg("true")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

/// 明示パス、または接続可能な候補鍵を返す。
pub fn resolve_identity_path(
    target: &SshTarget,
    cli_key: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(raw) = cli_key
        .map(str::to_string)
        .or_else(|| std::env::var("DEGIMA_CURSOR_SSH_KEY").ok())
        .or_else(|| std::env::var("SSH_KEY").ok())
    {
        let path = expand_home(raw.trim());
        if !path.is_file() {
            return Err(format!("SSH key not found: {}", path.display()));
        }
        if !key_authenticates(target, &path) {
            return Err(format!(
                "SSH key was rejected by {}@{}:{}: {}",
                target.user,
                target.host,
                target.port,
                path.display()
            ));
        }
        return Ok(path);
    }

    let mut tried = Vec::new();
    for path in default_candidate_keys() {
        if !path.is_file() {
            continue;
        }
        if key_authenticates(target, &path) {
            return Ok(path);
        }
        tried.push(path.display().to_string());
    }

    if tried.is_empty() {
        return Err(
            "No SSH private key found. Tried ~/.ssh/id_ed25519, id_rsa, cursor2_id_ed25519, and degima-agent dashboard key."
                .to_string(),
        );
    }
    Err(format!(
        "No usable SSH key for {}@{}:{}. Tried: {}",
        target.user,
        target.host,
        target.port,
        tried.join(", ")
    ))
}
