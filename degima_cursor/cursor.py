"""Cursor IDE の探索と Remote-SSH 起動（degima-agent `open-in-cursor.js` 相当）。"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REMOTE_WORKSPACE_PATH = "/root"
CURSOR_COMMAND_NOT_FOUND = (
    "The cursor command was not found. Install Cursor and make sure cursor is on PATH."
)
DEFAULT_EXEC_PATH = "/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"


def remote_workspace_path() -> str:
    return REMOTE_WORKSPACE_PATH


def _cursor_executable_candidates() -> list[Path]:
    if sys.platform == "win32":
        candidates: list[Path] = []
        local = os.environ.get("LOCALAPPDATA")
        if local:
            install_root = Path(local) / "Programs" / "cursor"
            bin_dir = install_root / "resources" / "app" / "bin"
            candidates.append(bin_dir / "cursor.cmd")
            candidates.append(install_root / "Cursor.exe")
        candidates.append(Path("cursor.cmd"))
        return candidates
    return [
        Path("cursor"),
        Path("/usr/local/bin/cursor"),
        Path("/Applications/Cursor.app/Contents/Resources/app/bin/cursor"),
    ]


def _find_on_path(base_name: str) -> Path | None:
    search_path = os.environ.get("PATH", DEFAULT_EXEC_PATH)
    for directory in search_path.split(os.pathsep):
        if not directory:
            continue
        candidate = Path(directory) / base_name
        if candidate.is_file():
            return candidate
        if sys.platform == "win32":
            for ext in (".COM", ".EXE", ".BAT", ".CMD"):
                with_ext = Path(directory) / f"{base_name}{ext}"
                if with_ext.is_file():
                    return with_ext
    return None


def find_cursor_executable() -> Path | None:
    if sys.platform == "win32":
        local = os.environ.get("LOCALAPPDATA")
        if local:
            cursor_exe = Path(local) / "Programs" / "cursor" / "Cursor.exe"
            if cursor_exe.is_file():
                return cursor_exe

    for candidate in _cursor_executable_candidates():
        as_str = str(candidate)
        if "/" in as_str or "\\" in as_str:
            if candidate.is_file():
                return candidate
            continue
        found = _find_on_path(as_str)
        if found is not None:
            return found
    return None


def remote_folder_uri(ssh_host: str, remote_path: str) -> str:
    normalized = remote_path if remote_path.startswith("/") else f"/{remote_path}"
    return f"vscode-remote://ssh-remote+{ssh_host}{normalized}"


def build_cursor_remote_launch_args(ssh_host: str, remote_path: str) -> list[str]:
    if sys.platform in ("win32", "darwin"):
        return [
            "--folder-uri",
            remote_folder_uri(ssh_host, remote_path),
            "--new-window",
        ]
    return [
        "--remote",
        f"ssh-remote+{ssh_host}",
        remote_path,
    ]


def launch_cursor_with_remote_folder(ssh_host: str, remote_path: str) -> None:
    cursor_bin = find_cursor_executable()
    if cursor_bin is None:
        raise RuntimeError(CURSOR_COMMAND_NOT_FOUND)

    args = build_cursor_remote_launch_args(ssh_host, remote_path)
    try:
        kwargs: dict = {
            "args": [str(cursor_bin), *args],
            "stdin": subprocess.DEVNULL,
            "stdout": subprocess.DEVNULL,
            "stderr": subprocess.DEVNULL,
        }
        if sys.platform != "win32":
            kwargs["start_new_session"] = True
        else:
            # DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
            kwargs["creationflags"] = 0x00000008 | 0x00000200
        subprocess.Popen(**kwargs)
    except OSError as exc:
        raise RuntimeError(f"Failed to launch Cursor: {exc}") from exc
