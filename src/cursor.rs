//! Cursor IDE の探索と Remote-SSH 起動（degima-agent `open-in-cursor.js` 相当）。

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const REMOTE_WORKSPACE_PATH: &str = "/root";
const CURSOR_COMMAND_NOT_FOUND: &str =
    "The cursor command was not found. Install Cursor and make sure cursor is on PATH.";

const DEFAULT_EXEC_PATH: &str = "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin";

pub fn remote_workspace_path() -> &'static str {
    REMOTE_WORKSPACE_PATH
}

fn file_exists(path: &Path) -> bool {
    path.is_file()
}

fn cursor_executable_candidates() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        let mut candidates = Vec::new();
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let install_root = PathBuf::from(local).join("Programs").join("cursor");
            let bin_dir = install_root.join("resources").join("app").join("bin");
            candidates.push(bin_dir.join("cursor.cmd"));
            candidates.push(install_root.join("Cursor.exe"));
        }
        candidates.push(PathBuf::from("cursor.cmd"));
        candidates
    } else {
        vec![
            PathBuf::from("cursor"),
            PathBuf::from("/usr/local/bin/cursor"),
            PathBuf::from("/Applications/Cursor.app/Contents/Resources/app/bin/cursor"),
        ]
    }
}

fn find_on_path(base_name: &str) -> Option<PathBuf> {
    let search_path = env::var("PATH").unwrap_or_else(|_| DEFAULT_EXEC_PATH.to_string());
    for dir in env::split_paths(&search_path) {
        let candidate = dir.join(base_name);
        if file_exists(&candidate) {
            return Some(candidate);
        }
        #[cfg(target_os = "windows")]
        {
            for ext in [".COM", ".EXE", ".BAT", ".CMD"] {
                let with_ext = dir.join(format!("{base_name}{ext}"));
                if file_exists(&with_ext) {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}

pub fn find_cursor_executable() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let cursor_exe = PathBuf::from(local)
                .join("Programs")
                .join("cursor")
                .join("Cursor.exe");
            if file_exists(&cursor_exe) {
                return Some(cursor_exe);
            }
        }
    }

    for candidate in cursor_executable_candidates() {
        let as_str = candidate.to_string_lossy();
        if as_str.contains('/') || as_str.contains('\\') {
            if file_exists(&candidate) {
                return Some(candidate);
            }
            continue;
        }
        if let Some(found) = find_on_path(&as_str) {
            return Some(found);
        }
    }
    None
}

pub fn remote_folder_uri(ssh_host: &str, remote_path: &str) -> String {
    let normalized = if remote_path.starts_with('/') {
        remote_path.to_string()
    } else {
        format!("/{remote_path}")
    };
    format!("vscode-remote://ssh-remote+{ssh_host}{normalized}")
}

pub fn build_cursor_remote_launch_args(ssh_host: &str, remote_path: &str) -> Vec<String> {
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        vec![
            "--folder-uri".to_string(),
            remote_folder_uri(ssh_host, remote_path),
            "--new-window".to_string(),
        ]
    } else {
        vec![
            "--remote".to_string(),
            format!("ssh-remote+{ssh_host}"),
            remote_path.to_string(),
        ]
    }
}

pub fn launch_cursor_with_remote_folder(ssh_host: &str, remote_path: &str) -> Result<(), String> {
    let cursor_bin = find_cursor_executable().ok_or_else(|| CURSOR_COMMAND_NOT_FOUND.to_string())?;
    let args = build_cursor_remote_launch_args(ssh_host, remote_path);

    let mut command = Command::new(&cursor_bin);
    command
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Mirror Electron `detached: true` — new process group on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    match command.spawn() {
        Ok(child) => {
            // Do not wait; mirror Electron `unref()`. Parent exits shortly after.
            std::mem::forget(child);
            Ok(())
        }
        Err(error) => Err(format!("Failed to launch Cursor: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_uri_for_root() {
        assert_eq!(
            remote_folder_uri("degima-host-10008", "/root"),
            "vscode-remote://ssh-remote+degima-host-10008/root"
        );
    }
}
