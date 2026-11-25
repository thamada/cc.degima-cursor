"""SSH コマンド文字列の解析（degima-agent `settings.parseSshCmd` 相当）。"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SshTarget:
    user: str
    host: str
    port: int


def parse_ssh_cmd(raw: str) -> SshTarget:
    text = raw.strip()
    if not text.startswith("ssh "):
        raise ValueError("SSH command must start with 'ssh '.")

    parts = [p for p in text[4:].split() if p]
    user = "root"
    host = ""
    port = 22
    i = 0

    while i < len(parts):
        token = parts[i]
        if token == "-p":
            if i + 1 >= len(parts):
                raise ValueError(f"Invalid SSH command: {raw}")
            try:
                port = int(parts[i + 1])
            except ValueError as exc:
                raise ValueError(f"Invalid SSH command: {raw}") from exc
            if not 0 <= port <= 65535:
                raise ValueError(f"Invalid SSH command: {raw}")
            i += 2
            continue
        if "@" in token:
            left, right = token.split("@", 1)
            user = left if left else "root"
            host = right
        elif not host and not token.startswith("-"):
            host = token
        i += 1

    if not host:
        raise ValueError(f"Invalid SSH command: {raw}")

    return SshTarget(user=user, host=host, port=port)
