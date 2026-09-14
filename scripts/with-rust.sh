#!/usr/bin/env sh
# Run a command with the Rust toolchain on PATH.
#
# rustup installs to ~/.cargo/bin and normally patches your shell profile, but
# a profile is only read by *new* shells — and CI, editors and desktop launchers
# often read none at all. Rather than depend on the caller's environment, every
# script that needs cargo goes through here.
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "openlogi: cargo not found." >&2
  echo "  Install the Rust toolchain:  https://rustup.rs" >&2
  echo "  Then re-run this command." >&2
  exit 127
fi

exec "$@"
