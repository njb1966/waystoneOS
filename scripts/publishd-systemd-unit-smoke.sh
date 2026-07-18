#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
verify_root="${WAYSTONE_PUBLISHD_SYSTEMD_VERIFY_ROOT:-/tmp/waystone-publishd-systemd-verify-$$}"
daemon_path="$verify_root/usr/bin/waystone-publishd"
unit_path="$verify_root/systemd/waystone-publishd.service"

mkdir -p "$verify_root/usr/bin"
mkdir -p "$verify_root/systemd"

printf '#!/usr/bin/env sh\nexit 0\n' > "$daemon_path"
sed "s#ExecStart=/usr/bin/waystone-publishd#ExecStart=$daemon_path#" \
  "$repo_root/services/publishd/systemd/waystone-publishd.service" > "$unit_path"
chmod 755 "$daemon_path"
chmod 644 "$unit_path"

systemd-analyze verify "$unit_path"

echo "publishd systemd unit smoke: syntax verified"
