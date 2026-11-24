//! SSH コマンド文字列の解析（degima-agent `settings.parseSshCmd` 相当）。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    pub user: String,
    pub host: String,
    pub port: u16,
}

pub fn parse_ssh_cmd(raw: &str) -> Result<SshTarget, String> {
    let text = raw.trim();
    if !text.starts_with("ssh ") {
        return Err("SSH command must start with 'ssh '.".to_string());
    }

    let parts: Vec<&str> = text[4..]
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .collect();

    let mut user = "root".to_string();
    let mut host = String::new();
    let mut port: u16 = 22;
    let mut i = 0usize;

    while i < parts.len() {
        let token = parts[i];
        if token == "-p" {
            let Some(port_text) = parts.get(i + 1) else {
                return Err(format!("Invalid SSH command: {raw}"));
            };
            port = port_text
                .parse::<u16>()
                .map_err(|_| format!("Invalid SSH command: {raw}"))?;
            i += 2;
            continue;
        }
        if token.contains('@') {
            let mut split = token.splitn(2, '@');
            let left = split.next().unwrap_or("root");
            let right = split.next().unwrap_or("");
            user = if left.is_empty() {
                "root".to_string()
            } else {
                left.to_string()
            };
            host = right.to_string();
        } else if host.is_empty() && !token.starts_with('-') {
            host = token.to_string();
        }
        i += 1;
    }

    if host.is_empty() {
        return Err(format!("Invalid SSH command: {raw}"));
    }

    Ok(SshTarget { user, host, port })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_host_port() {
        let target = parse_ssh_cmd("ssh root@dk11-kusaka24.degima.ai -p 10008").unwrap();
        assert_eq!(target.user, "root");
        assert_eq!(target.host, "dk11-kusaka24.degima.ai");
        assert_eq!(target.port, 10008);
    }

    #[test]
    fn defaults_port_and_user() {
        let target = parse_ssh_cmd("ssh example.degima.ai").unwrap();
        assert_eq!(target.user, "root");
        assert_eq!(target.host, "example.degima.ai");
        assert_eq!(target.port, 22);
    }
}
