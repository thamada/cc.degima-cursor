//! degima-cursor — DEGIMA Device を Cursor Remote-SSH で開く CLI（AI Agent 相当）。

mod cursor;
mod ssh_cmd;
mod ssh_config;
mod ssh_key;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use cursor::{launch_cursor_with_remote_folder, remote_workspace_path};
use ssh_cmd::parse_ssh_cmd;
use ssh_config::ensure_device_ssh_config;
use ssh_key::resolve_identity_path;

fn usage() -> ! {
    eprintln!(
        "Usage:
  degima-cursor [\"ssh user@host -p PORT\"]
  degima-cursor --ssh-key PATH [\"ssh user@host -p PORT\"]

If the SSH command is omitted, you will be prompted.
Example: ssh root@dk11-kusaka24.degima.ai -p 10008

Without --ssh-key, a private key that can authenticate to the host is
auto-selected (id_ed25519, id_rsa, cursor2_id_ed25519, degima-agent key)."
    );
    process::exit(2);
}

fn read_ssh_command_interactive() -> Result<String, String> {
    print!("SSH command: ");
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {e}"))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read SSH command: {e}"))?;
    let text = line.trim().to_string();
    if text.is_empty() {
        return Err("SSH command is required.".to_string());
    }
    Ok(text)
}

fn parse_args(args: &[String]) -> Result<(Option<String>, Option<String>), String> {
    let mut ssh_cmd: Option<String> = None;
    let mut ssh_key: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => usage(),
            "--ssh-key" => {
                let Some(value) = args.get(i + 1) else {
                    return Err("--ssh-key requires a path.".to_string());
                };
                ssh_key = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown option: {flag}"));
            }
            value => {
                if ssh_cmd.is_some() {
                    return Err("Only one SSH command argument is allowed.".to_string());
                }
                ssh_cmd = Some(value.to_string());
                i += 1;
            }
        }
    }
    Ok((ssh_cmd, ssh_key))
}

fn open_in_cursor(ssh_cmd_raw: &str, identity_path: &PathBuf) -> Result<String, String> {
    let target = parse_ssh_cmd(ssh_cmd_raw)?;
    let ensured = ensure_device_ssh_config(&target, identity_path)
        .map_err(|e| format!("Failed to update SSH config: {e}"))?;
    launch_cursor_with_remote_folder(&ensured.ssh_host, remote_workspace_path())?;
    Ok(format!(
        "Opening {} on {}@{}:{} in Cursor.\nUsing SSH key: {}",
        remote_workspace_path(),
        ensured.user,
        ensured.host,
        ensured.port,
        identity_path.display()
    ))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (ssh_cmd_arg, ssh_key_arg) = match parse_args(&args) {
        Ok(v) => v,
        Err(error) => {
            eprintln!("error: {error}");
            usage();
        }
    };

    let ssh_cmd = match ssh_cmd_arg {
        Some(cmd) => cmd,
        None => match read_ssh_command_interactive() {
            Ok(cmd) => cmd,
            Err(error) => {
                eprintln!("error: {error}");
                process::exit(1);
            }
        },
    };

    let target = match parse_ssh_cmd(&ssh_cmd) {
        Ok(t) => t,
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    };

    let identity_path = match resolve_identity_path(&target, ssh_key_arg.as_deref()) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    };

    match open_in_cursor(&ssh_cmd, &identity_path) {
        Ok(message) => {
            println!("{message}");
        }
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}
