"""接続先に通る SSH 秘密鍵の解決。"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from .ssh_cmd import SshTarget
from .ssh_config import expand_home, home_dir


def _default_candidate_keys() -> list[Path]:
    keys = [
        expand_home("~/.ssh/id_ed25519"),
        expand_home("~/.ssh/id_rsa"),
        expand_home("~/.ssh/cursor2_id_ed25519"),
    ]

    if sys.platform == "darwin":
        keys.append(
            home_dir()
            / "Library"
            / "Application Support"
            / "degima-agent"
            / "ssh-keys"
            / "degima_dashboard_ed25519"
        )
    elif sys.platform.startswith("linux"):
        keys.append(
            expand_home("~/.config/degima-agent/ssh-keys/degima_dashboard_ed25519")
        )
    elif sys.platform == "win32":
        appdata = os.environ.get("APPDATA")
        if appdata:
            keys.append(
                Path(appdata)
                / "degima-agent"
                / "ssh-keys"
                / "degima_dashboard_ed25519"
            )

    return keys


def _key_authenticates(target: SshTarget, identity_path: Path) -> bool:
    try:
        result = subprocess.run(
            [
                "ssh",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-o",
                "IdentitiesOnly=yes",
                "-i",
                str(identity_path),
                "-p",
                str(target.port),
                f"{target.user}@{target.host}",
                "true",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    except OSError:
        return False
    return result.returncode == 0


def resolve_identity_path(
    target: SshTarget, cli_key: str | None = None
) -> Path:
    explicit = (
        cli_key
        or os.environ.get("DEGIMA_CURSOR_SSH_KEY")
        or os.environ.get("SSH_KEY")
    )
    if explicit:
        path = expand_home(explicit.strip())
        if not path.is_file():
            raise ValueError(f"SSH key not found: {path}")
        if not _key_authenticates(target, path):
            raise ValueError(
                f"SSH key was rejected by {target.user}@{target.host}:{target.port}: {path}"
            )
        return path

    tried: list[str] = []
    for path in _default_candidate_keys():
        if not path.is_file():
            continue
        if _key_authenticates(target, path):
            return path
        tried.append(str(path))

    if not tried:
        raise ValueError(
            "No SSH private key found. Tried ~/.ssh/id_ed25519, id_rsa, "
            "cursor2_id_ed25519, and degima-agent dashboard key."
        )
    raise ValueError(
        f"No usable SSH key for {target.user}@{target.host}:{target.port}. "
        f"Tried: {', '.join(tried)}"
    )
