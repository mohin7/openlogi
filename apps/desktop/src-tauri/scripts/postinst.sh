#!/bin/sh
# Post-install setup for OpenLogi.
#
# Everything here is the system configuration OpenLogi needs in order to run
# *without* privileges afterwards: a udev rule granting the logged-in user
# access to Logitech HID++ endpoints, and a dedicated group for uinput.
set -e

# --- uinput group ------------------------------------------------------------
# A dedicated group rather than "input": membership of "input" grants *read*
# access to every /dev/input/event* device, i.e. the ability to log every
# keystroke on the system. OpenLogi only needs to *write* to uinput.
if ! getent group openlogi >/dev/null 2>&1; then
  groupadd --system openlogi
fi

# Add the human who installed the package, where we can identify them.
TARGET_USER="${SUDO_USER:-${PKEXEC_UID:+$(id -nu "$PKEXEC_UID" 2>/dev/null)}}"
if [ -n "$TARGET_USER" ] && [ "$TARGET_USER" != "root" ]; then
  if ! id -nG "$TARGET_USER" 2>/dev/null | tr ' ' '\n' | grep -qx openlogi; then
    usermod -aG openlogi "$TARGET_USER" || true
    echo "OpenLogi: added '$TARGET_USER' to group 'openlogi'."
    echo "OpenLogi: log out and back in for this to take effect."
  fi
fi

# --- uinput module -----------------------------------------------------------
modprobe uinput >/dev/null 2>&1 || true
if [ ! -f /etc/modules-load.d/openlogi.conf ]; then
  echo uinput > /etc/modules-load.d/openlogi.conf
fi

# --- apply udev rules --------------------------------------------------------
# The rules file ships at /usr/lib/udev/rules.d/70-openlogi.rules. The 70
# prefix is load-bearing: systemd's 73-seat-late.rules runs the uaccess
# builtin, so a higher-numbered file would set TAG+="uaccess" too late and the
# ACL would silently never be applied.
if command -v udevadm >/dev/null 2>&1; then
  udevadm control --reload-rules || true
  udevadm trigger --subsystem-match=hidraw --action=add || true
  udevadm trigger --subsystem-match=misc --action=add || true
fi

exit 0
