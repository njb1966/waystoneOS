#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
verify_root="${WAYSTONE_AUDIOD_SYSTEMD_VERIFY_ROOT:-/tmp/waystone-audiod-systemd-verify-$$}"
daemon_path="$verify_root/usr/bin/waystone-audiod"
unit_path="$verify_root/systemd/waystone-audiod.service"

mkdir -p "$verify_root/usr/bin"
mkdir -p "$verify_root/systemd"

printf '#!/usr/bin/env sh\nexit 0\n' > "$daemon_path"
sed "s#ExecStart=/usr/bin/waystone-audiod#ExecStart=$daemon_path#" \
  "$repo_root/services/audiod/systemd/waystone-audiod.service" > "$unit_path"
chmod 755 "$daemon_path"
chmod 644 "$unit_path"

systemd-analyze verify "$unit_path"

echo "audiod systemd unit smoke: syntax verified"
