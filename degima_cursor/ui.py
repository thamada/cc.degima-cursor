"""ターミナル向けの色付き表示（外部依存なし）。"""

from __future__ import annotations

import os
import sys
from pathlib import Path

from .ssh_cmd import SshTarget


def _color_enabled(stream=sys.stdout) -> bool:
    if os.environ.get("NO_COLOR"):
        return False
    if os.environ.get("TERM") == "dumb":
        return False
    return hasattr(stream, "isatty") and stream.isatty()


class _Ansi:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    CYAN = "\033[36m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    MAGENTA = "\033[35m"
    BLUE = "\033[34m"
    WHITE = "\033[97m"


def _c(code: str, text: str, enabled: bool) -> str:
    if not enabled:
        return text
    return f"{code}{text}{_Ansi.RESET}"


def print_parse_summary(
    target: SshTarget,
    identity_path: Path | None = None,
    ssh_host: str | None = None,
    remote_path: str | None = None,
) -> None:
    """解析・解決結果を Cursor 起動前にカラフル表示する。"""
    enabled = _color_enabled()
    width = 52
    title = " degima-cursor · parsed "

    top = "╭" + title + "─" * max(1, width - len(title) - 1) + "╮"
    bottom = "╰" + "─" * width + "╯"

    def row(label: str, value: str, value_color: str) -> str:
        label_s = _c(_Ansi.DIM, f"{label:<10}", enabled)
        value_s = _c(_Ansi.BOLD + value_color, value, enabled)
        return f"│ {label_s} {value_s}"

    lines = [
        _c(_Ansi.CYAN + _Ansi.BOLD, top, enabled),
        row("user", target.user, _Ansi.GREEN),
        row("host", target.host, _Ansi.YELLOW),
        row("port", str(target.port), _Ansi.MAGENTA),
    ]
    if identity_path is not None:
        lines.append(row("key", str(identity_path), _Ansi.BLUE))
    if ssh_host is not None:
        lines.append(row("alias", ssh_host, _Ansi.CYAN))
    if remote_path is not None:
        lines.append(row("path", remote_path, _Ansi.WHITE))
    lines.append(_c(_Ansi.CYAN + _Ansi.BOLD, bottom, enabled))
    lines.append(
        _c(_Ansi.DIM, "→ launching Cursor Remote-SSH…", enabled)
    )
    print("\n".join(lines), flush=True)
