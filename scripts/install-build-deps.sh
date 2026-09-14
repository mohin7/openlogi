#!/usr/bin/env bash
# System packages needed to BUILD OpenLogi. End users installing the AppImage
# need none of these.
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Re-run: sudo $0" >&2
  exit 1
fi

if command -v apt-get >/dev/null; then
  apt-get update
  apt-get install -y --no-install-recommends \
    build-essential pkg-config curl file \
    libudev-dev libssl-dev \
    libgtk-3-dev librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev
elif command -v dnf >/dev/null; then
  dnf install -y \
    gcc gcc-c++ make pkgconf-pkg-config \
    systemd-devel openssl-devel \
    gtk3-devel librsvg2-devel \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel
elif command -v pacman >/dev/null; then
  pacman -S --needed --noconfirm \
    base-devel pkgconf systemd-libs openssl \
    gtk3 librsvg webkit2gtk-4.1 libayatana-appindicator
else
  echo "Unsupported package manager. Needed: webkit2gtk 4.1, gtk3, libudev, openssl, librsvg, ayatana-appindicator." >&2
  exit 1
fi

echo "build dependencies installed"
