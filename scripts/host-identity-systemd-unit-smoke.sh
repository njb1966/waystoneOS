#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
verify_root="${WAYSTONE_HOST_IDENTITY_SYSTEMD_VERIFY_ROOT:-/tmp/waystone-host-identity-systemd-verify-$$}"
hostd_path="$verify_root/usr/bin/waystone-hostd"
identityd_path="$verify_root/usr/bin/waystone-identityd"
host_unit_path="$verify_root/systemd/waystone-hostd.service"
identity_unit_path="$verify_root/systemd/waystone-identityd.service"

mkdir -p "$verify_root/usr/bin"
mkdir -p "$verify_root/systemd"

printf '#!/usr/bin/env sh\nexit 0\n' > "$hostd_path"
printf '#!/usr/bin/env sh\nexit 0\n' > "$identityd_path"
sed "s#ExecStart=/usr/bin/waystone-hostd#ExecStart=$hostd_path#" \
  "$repo_root/services/hostd/systemd/waystone-hostd.service" > "$host_unit_path"
sed "s#ExecStart=/usr/bin/waystone-identityd#ExecStart=$identityd_path#" \
  "$repo_root/services/identityd/systemd/waystone-identityd.service" > "$identity_unit_path"
chmod 755 "$hostd_path"
chmod 755 "$identityd_path"
chmod 644 "$host_unit_path"
chmod 644 "$identity_unit_path"

systemd-analyze verify "$host_unit_path" "$identity_unit_path"

echo "host/identity systemd unit smoke: syntax verified"
