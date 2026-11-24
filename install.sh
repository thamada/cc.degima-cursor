#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_DIR="${SCRIPT_DIR}/.local"
CARGO_HOME="${LOCAL_DIR}/cargo"
RUSTUP_HOME="${LOCAL_DIR}/rustup"
RUST_VERSION="${RUST_VERSION:-stable}"

case "$(uname -s)" in
  Darwin|Linux) ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac

mkdir -p "${LOCAL_DIR}"
export CARGO_HOME
export RUSTUP_HOME
export PATH="${CARGO_HOME}/bin:${PATH}"

if [[ ! -x "${CARGO_HOME}/bin/rustup" ]]; then
  echo "Installing rustup into ${LOCAL_DIR}..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain "${RUST_VERSION}"
else
  echo "rustup already present at ${CARGO_HOME}/bin/rustup"
  "${CARGO_HOME}/bin/rustup" toolchain install "${RUST_VERSION}"
  "${CARGO_HOME}/bin/rustup" default "${RUST_VERSION}"
fi

cat > "${LOCAL_DIR}/env.sh" <<EOF
export CARGO_HOME="${CARGO_HOME}"
export RUSTUP_HOME="${RUSTUP_HOME}"
export PATH="${CARGO_HOME}/bin:\${PATH}"
EOF

echo "Using rustc: $("${CARGO_HOME}/bin/rustc" --version)"
echo "Using cargo: $("${CARGO_HOME}/bin/cargo" --version)"
echo
echo "Setup complete."
echo "Run with:"
echo "  cd ${SCRIPT_DIR}"
echo "  make run"
