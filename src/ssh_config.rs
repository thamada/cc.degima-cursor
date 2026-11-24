//! `~/.ssh/config` の Device 別ブロック更新（degima-agent `open-in-cursor.js` 相当）。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::ssh_cmd::SshTarget;

const SSH_CONFIG_MARKER_BEGIN: &str = "# BEGIN degima-terminal rocket";
const SSH_CONFIG_MARKER_END: &str = "# END degima-terminal rocket";

#[derive(Debug, Clone)]
pub struct EnsuredSshConfig {
    pub ssh_host: String,
    pub user: String,
    pub host: String,
    pub port: u16,
}

pub fn expand_home(value: &str) -> PathBuf {
    if value == "~" {
        return home_dir();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    if let Some(rest) = value.strip_prefix("~\\") {
        return home_dir().join(rest);
    }
    PathBuf::from(value)
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("HOME or USERPROFILE must be set")
}

pub fn ssh_config_path() -> PathBuf {
    home_dir().join(".ssh").join("config")
}

pub fn device_ssh_host_alias(host: &str, port: u16) -> String {
    let safe_host: String = host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("degima-{safe_host}-{port}")
}

fn device_ssh_config_markers(host: &str, port: u16) -> (String, String) {
    let marker_id = format!("{host}:{port}");
    (
        format!("{SSH_CONFIG_MARKER_BEGIN} {marker_id}"),
        format!("{SSH_CONFIG_MARKER_END} {marker_id}"),
    )
}

fn quote_ssh_config_value(value: &str) -> String {
    if value.chars().any(|c| c.is_whitespace() || c == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn build_device_ssh_config_block(
    ssh_host: &str,
    host_name: &str,
    user: &str,
    port: u16,
    identity_path: &Path,
    begin: &str,
    end: &str,
) -> String {
    let identity = quote_ssh_config_value(&identity_path.display().to_string());
    [
        begin.to_string(),
        format!("Host {ssh_host}"),
        format!("  HostName {host_name}"),
        format!("  User {user}"),
        format!("  Port {port}"),
        format!("  IdentityFile {identity}"),
        "  IdentitiesOnly yes".to_string(),
        "  StrictHostKeyChecking accept-new".to_string(),
        end.to_string(),
    ]
    .join("\n")
}

fn upsert_marked_ssh_config_block(existing_text: &str, begin: &str, end: &str, block: &str) -> String {
    if let (Some(begin_index), Some(end_index)) =
        (existing_text.find(begin), existing_text.find(end))
    {
        if end_index > begin_index {
            let before = existing_text[..begin_index].trim_end();
            let after = existing_text[end_index + end.len()..].trim_start();
            let mut chunks = Vec::new();
            if !before.is_empty() {
                chunks.push(before.to_string());
            }
            chunks.push(block.to_string());
            if !after.is_empty() {
                chunks.push(after.to_string());
            }
            return format!("{}\n", chunks.join("\n\n"));
        }
    }

    let trimmed = existing_text.trim_end();
    if trimmed.is_empty() {
        format!("{block}\n")
    } else {
        format!("{trimmed}\n\n{block}\n")
    }
}

fn remove_legacy_hostless_ssh_block(existing_text: &str) -> String {
    let Some(begin_index) = existing_text.find(SSH_CONFIG_MARKER_BEGIN) else {
        return existing_text.to_string();
    };

    let begin_line_end = existing_text[begin_index..]
        .find('\n')
        .map(|i| begin_index + i)
        .unwrap_or(existing_text.len());
    let begin_line = &existing_text[begin_index..begin_line_end];
    if begin_line != SSH_CONFIG_MARKER_BEGIN {
        return existing_text.to_string();
    }

    let Some(rel_end) = existing_text[begin_index..].find(SSH_CONFIG_MARKER_END) else {
        return existing_text.to_string();
    };
    let end_index = begin_index + rel_end;

    let before = existing_text[..begin_index].trim_end();
    let after = existing_text[end_index + SSH_CONFIG_MARKER_END.len()..].trim_start();
    let mut chunks = Vec::new();
    if !before.is_empty() {
        chunks.push(before);
    }
    if !after.is_empty() {
        chunks.push(after);
    }
    if chunks.is_empty() {
        String::new()
    } else {
        format!("{}\n", chunks.join("\n\n"))
    }
}

#[cfg(unix)]
fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn chmod(_path: &Path, _mode: u32) {}

pub fn ensure_device_ssh_config(
    target: &SshTarget,
    identity_path: &Path,
) -> io::Result<EnsuredSshConfig> {
    let ssh_host = device_ssh_host_alias(&target.host, target.port);
    let (begin, end) = device_ssh_config_markers(&target.host, target.port);
    let ssh_dir = home_dir().join(".ssh");
    fs::create_dir_all(&ssh_dir)?;
    chmod(&ssh_dir, 0o700);

    let config_path = ssh_config_path();
    let mut existing_text = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    existing_text = remove_legacy_hostless_ssh_block(&existing_text);
    let block = build_device_ssh_config_block(
        &ssh_host,
        &target.host,
        &target.user,
        target.port,
        identity_path,
        &begin,
        &end,
    );
    let next_text = upsert_marked_ssh_config_block(&existing_text, &begin, &end, &block);
    fs::write(&config_path, next_text)?;
    chmod(&config_path, 0o600);

    Ok(EnsuredSshConfig {
        ssh_host,
        user: target.user.clone(),
        host: target.host.clone(),
        port: target.port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_sanitizes_host() {
        assert_eq!(
            device_ssh_host_alias("dk11-kusaka24.degima.ai", 10008),
            "degima-dk11-kusaka24.degima.ai-10008"
        );
    }

    #[test]
    fn upsert_replaces_existing_block() {
        let begin = "# BEGIN degima-terminal rocket h:1";
        let end = "# END degima-terminal rocket h:1";
        let existing = format!("alpha\n\n{begin}\nold\n{end}\n\nbeta\n");
        let next = upsert_marked_ssh_config_block(&existing, begin, end, "NEW");
        assert!(next.contains("NEW"));
        assert!(!next.contains("old"));
        assert!(next.contains("alpha"));
        assert!(next.contains("beta"));
    }
}
