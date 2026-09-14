#!/usr/bin/env bash
# One-shot developer setup. Runs the two privileged steps together so you are
# prompted for your password exactly once.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> Installing build dependencies and udev rules (needs sudo)"
sudo "$HERE/install-build-deps.sh"
sudo "$HERE/install-udev-rules.sh"

if ! command -v cargo >/dev/null; then
  echo "==> Installing Rust"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

echo "==> Building the probe"
cargo build --release -p logi-probe

echo
echo "Done. Now run:  ./target/release/logi-probe"
