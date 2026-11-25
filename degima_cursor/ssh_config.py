"""`~/.ssh/config` の Device 別ブロック更新（degima-agent `open-in-cursor.js` 相当）。"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

from .ssh_cmd import SshTarget

SSH_CONFIG_MARKER_BEGIN = "# BEGIN degima-terminal rocket"
SSH_CONFIG_MARKER_END = "# END degima-terminal rocket"


@dataclass(frozen=True)
class EnsuredSshConfig:
    ssh_host: str
    user: str
    host: str
    port: int


def home_dir() -> Path:
    home = os.environ.get("HOME") or os.environ.get("USERPROFILE")
    if not home:
        raise RuntimeError("HOME or USERPROFILE must be set")
    return Path(home)


def expand_home(value: str) -> Path:
    if value == "~":
        return home_dir()
    if value.startswith("~/"):
        return home_dir() / value[2:]
    if value.startswith("~\\"):
        return home_dir() / value[2:]
    return Path(value)


def ssh_config_path() -> Path:
    return home_dir() / ".ssh" / "config"


def device_ssh_host_alias(host: str, port: int) -> str:
    safe_host = "".join(
        c if (c.isalnum() or c in ".-") else "-" for c in host
    )
    return f"degima-{safe_host}-{port}"


def _device_ssh_config_markers(host: str, port: int) -> tuple[str, str]:
    marker_id = f"{host}:{port}"
    return (
        f"{SSH_CONFIG_MARKER_BEGIN} {marker_id}",
        f"{SSH_CONFIG_MARKER_END} {marker_id}",
    )


def _quote_ssh_config_value(value: str) -> str:
    if any(c.isspace() or c == '"' for c in value):
        return '"' + value.replace('"', '\\"') + '"'
    return value


def _build_device_ssh_config_block(
    ssh_host: str,
    host_name: str,
    user: str,
    port: int,
    identity_path: Path,
    begin: str,
    end: str,
) -> str:
    identity = _quote_ssh_config_value(str(identity_path))
    return "\n".join(
        [
            begin,
            f"Host {ssh_host}",
            f"  HostName {host_name}",
            f"  User {user}",
            f"  Port {port}",
            f"  IdentityFile {identity}",
            "  IdentitiesOnly yes",
            "  StrictHostKeyChecking accept-new",
            end,
        ]
    )


def _upsert_marked_ssh_config_block(
    existing_text: str, begin: str, end: str, block: str
) -> str:
    begin_index = existing_text.find(begin)
    end_index = existing_text.find(end)
    if begin_index != -1 and end_index != -1 and end_index > begin_index:
        before = existing_text[:begin_index].rstrip()
        after = existing_text[end_index + len(end) :].lstrip()
        chunks: list[str] = []
        if before:
            chunks.append(before)
        chunks.append(block)
        if after:
            chunks.append(after)
        return "\n\n".join(chunks) + "\n"

    trimmed = existing_text.rstrip()
    if not trimmed:
        return f"{block}\n"
    return f"{trimmed}\n\n{block}\n"


def _remove_legacy_hostless_ssh_block(existing_text: str) -> str:
    begin_index = existing_text.find(SSH_CONFIG_MARKER_BEGIN)
    if begin_index == -1:
        return existing_text

    begin_line_end = existing_text.find("\n", begin_index)
    if begin_line_end == -1:
        begin_line_end = len(existing_text)
    begin_line = existing_text[begin_index:begin_line_end]
    if begin_line != SSH_CONFIG_MARKER_BEGIN:
        return existing_text

    rel_end = existing_text.find(SSH_CONFIG_MARKER_END, begin_index)
    if rel_end == -1:
        return existing_text

    before = existing_text[:begin_index].rstrip()
    after = existing_text[rel_end + len(SSH_CONFIG_MARKER_END) :].lstrip()
    chunks: list[str] = []
    if before:
        chunks.append(before)
    if after:
        chunks.append(after)
    if not chunks:
        return ""
    return "\n\n".join(chunks) + "\n"


def _chmod(path: Path, mode: int) -> None:
    if os.name == "nt":
        return
    try:
        os.chmod(path, mode)
    except OSError:
        pass


def ensure_device_ssh_config(
    target: SshTarget, identity_path: Path
) -> EnsuredSshConfig:
    ssh_host = device_ssh_host_alias(target.host, target.port)
    begin, end = _device_ssh_config_markers(target.host, target.port)
    ssh_dir = home_dir() / ".ssh"
    ssh_dir.mkdir(parents=True, exist_ok=True)
    _chmod(ssh_dir, 0o700)

    config_path = ssh_config_path()
    if config_path.exists():
        existing_text = config_path.read_text(encoding="utf-8")
    else:
        existing_text = ""

    existing_text = _remove_legacy_hostless_ssh_block(existing_text)
    block = _build_device_ssh_config_block(
        ssh_host,
        target.host,
        target.user,
        target.port,
        identity_path,
        begin,
        end,
    )
    next_text = _upsert_marked_ssh_config_block(existing_text, begin, end, block)
    config_path.write_text(next_text, encoding="utf-8")
    _chmod(config_path, 0o600)

    return EnsuredSshConfig(
        ssh_host=ssh_host,
        user=target.user,
        host=target.host,
        port=target.port,
    )
