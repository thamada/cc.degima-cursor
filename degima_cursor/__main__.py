"""degima-cursor CLI エントリポイント。"""

from __future__ import annotations

import sys
from pathlib import Path

from .cursor import launch_cursor_with_remote_folder, remote_workspace_path
from .ssh_cmd import parse_ssh_cmd
from .ssh_config import ensure_device_ssh_config
from .ssh_key import resolve_identity_path


def usage() -> None:
    print(
        """Usage:
  degima-cursor ["ssh user@host -p PORT"]
  degima-cursor --ssh-key PATH ["ssh user@host -p PORT"]

If the SSH command is omitted, you will be prompted.
Example: ssh root@dk11-kusaka24.degima.ai -p 10008

Without --ssh-key, a private key that can authenticate to the host is
auto-selected (id_ed25519, id_rsa, cursor2_id_ed25519, degima-agent key).""",
        file=sys.stderr,
    )
    raise SystemExit(2)


def read_ssh_command_interactive() -> str:
    try:
        text = input("SSH command: ").strip()
    except EOFError as exc:
        raise ValueError("Failed to read SSH command.") from exc
    if not text:
        raise ValueError("SSH command is required.")
    return text


def parse_args(args: list[str]) -> tuple[str | None, str | None]:
    ssh_cmd: str | None = None
    ssh_key: str | None = None
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in ("-h", "--help"):
            usage()
        if arg == "--ssh-key":
            if i + 1 >= len(args):
                raise ValueError("--ssh-key requires a path.")
            ssh_key = args[i + 1]
            i += 2
            continue
        if arg.startswith("-"):
            raise ValueError(f"Unknown option: {arg}")
        if ssh_cmd is not None:
            raise ValueError("Only one SSH command argument is allowed.")
        ssh_cmd = arg
        i += 1
    return ssh_cmd, ssh_key


def open_in_cursor(ssh_cmd_raw: str, identity_path: Path) -> str:
    target = parse_ssh_cmd(ssh_cmd_raw)
    ensured = ensure_device_ssh_config(target, identity_path)
    launch_cursor_with_remote_folder(ensured.ssh_host, remote_workspace_path())
    return (
        f"Opening {remote_workspace_path()} on "
        f"{ensured.user}@{ensured.host}:{ensured.port} in Cursor.\n"
        f"Using SSH key: {identity_path}"
    )


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    try:
        ssh_cmd_arg, ssh_key_arg = parse_args(args)
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        usage()

    try:
        ssh_cmd = ssh_cmd_arg if ssh_cmd_arg is not None else read_ssh_command_interactive()
        target = parse_ssh_cmd(ssh_cmd)
        identity_path = resolve_identity_path(target, ssh_key_arg)
        message = open_in_cursor(ssh_cmd, identity_path)
    except (ValueError, OSError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(message)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
