#!/usr/bin/env bash
# Install OpenLogi's udev rules. This is the ONLY step that needs root, and it
# is a one-time install action — OpenLogi itself never runs privileged.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULES_SRC="$HERE/70-openlogi.rules"
RULES_DST="/etc/udev/rules.d/70-openlogi.rules"
LEGACY_DST="/etc/udev/rules.d/99-openlogi.rules"

if [[ $EUID -ne 0 ]]; then
  echo "This script needs root to write ${RULES_DST}." >&2
  echo "Re-run: sudo $0" >&2
  exit 1
fi

# The user who invoked sudo — the one who actually needs the access.
TARGET_USER="${SUDO_USER:-${PKEXEC_UID:+$(id -nu "$PKEXEC_UID")}}"
if [[ -z "${TARGET_USER:-}" || "$TARGET_USER" == "root" ]]; then
  echo "warning: could not determine the desktop user; skipping group setup." >&2
  TARGET_USER=""
fi

# Earlier builds shipped this at 99-, which is too late for the uaccess builtin
# in 73-seat-late.rules. Remove it so the two do not both apply.
if [[ -f "$LEGACY_DST" ]]; then
  rm -f "$LEGACY_DST"
  echo "removed stale ${LEGACY_DST}"
fi

install -m 0644 "$RULES_SRC" "$RULES_DST"
echo "installed ${RULES_DST}"

# Dedicated group for uinput write access (see the rules file for why this is
# not the "input" group).
if ! getent group openlogi >/dev/null; then
  groupadd --system openlogi
  echo "created group 'openlogi'"
fi

ADDED_GROUP=0
if [[ -n "$TARGET_USER" ]]; then
  if id -nG "$TARGET_USER" | tr ' ' '\n' | grep -qx openlogi; then
    echo "user '$TARGET_USER' is already in group 'openlogi'"
  else
    usermod -aG openlogi "$TARGET_USER"
    echo "added '$TARGET_USER' to group 'openlogi'"
    ADDED_GROUP=1
  fi
fi

# Make sure uinput is present now and on every boot.
modprobe uinput 2>/dev/null || true
if [[ ! -f /etc/modules-load.d/openlogi.conf ]]; then
  echo uinput > /etc/modules-load.d/openlogi.conf
  echo "enabled uinput at boot"
fi

udevadm control --reload-rules
# Re-apply to devices that are already plugged in, so no replug is needed.
udevadm trigger --subsystem-match=hidraw --action=add
udevadm trigger --subsystem-match=misc --action=add
echo "udev rules reloaded and applied"

echo
echo "Verify:"
echo "  getfacl -p /dev/hidraw6 | grep '^user:'   # expect a user ACL entry"
echo "  ls -l /dev/uinput                         # expect group 'openlogi'"
if [[ $ADDED_GROUP -eq 1 ]]; then
  echo
  echo "NOTE: group membership only applies to NEW logins."
  echo "      Log out and back in, or start the app with: newgrp openlogi"
fi
