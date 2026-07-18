#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
verify_root="${WAYSTONE_PROJECTD_SYSTEMD_VERIFY_ROOT:-/tmp/waystone-projectd-systemd-verify-$$}"
daemon_path="$verify_root/usr/bin/waystone-projectd"
unit_path="$verify_root/systemd/waystone-projectd.service"

mkdir -p "$verify_root/usr/bin"
mkdir -p "$verify_root/systemd"

printf '#!/usr/bin/env sh\nexit 0\n' > "$daemon_path"
sed "s#ExecStart=/usr/bin/waystone-projectd#ExecStart=$daemon_path#" \
  "$repo_root/services/projectd/systemd/waystone-projectd.service" > "$unit_path"
chmod 755 "$daemon_path"
chmod 644 "$unit_path"

systemd-analyze verify "$unit_path"

echo "projectd systemd unit smoke: syntax verified"
