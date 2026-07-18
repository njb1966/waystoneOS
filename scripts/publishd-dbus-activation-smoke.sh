#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
activation_root="${WAYSTONE_PUBLISHD_DBUS_ACTIVATION_ROOT:-/tmp/waystone-publishd-dbus-activation-$$}"
service_dir="$activation_root/dbus-1/services"
service_file="$service_dir/org.waystone.Publish1.service"
daemon_path="$repo_root/target/debug/waystone-publishd"

cd "$repo_root"

cargo build -p waystone-publishd

mkdir -p "$service_dir"

{
  printf "[D-BUS Service]\n"
  printf "Name=org.waystone.Publish1\n"
  printf "Exec=%s\n" "$daemon_path"
} > "$service_file"

export WAYSTONE_PUBLISHD_REPO_ROOT="$repo_root"

XDG_DATA_HOME="$activation_root" dbus-run-session -- bash -c '
set -eu

preview_output="$(busctl --user call \
  org.waystone.Publish1 \
  /org/waystone/Publish \
  org.waystone.Publish1 \
  PreviewPublication \
  s "{\"project_path\":\"$WAYSTONE_PUBLISHD_REPO_ROOT/examples/projects/ssh-capsule.wayproject\",\"target\":\"production\",\"hosts_root\":\"$WAYSTONE_PUBLISHD_REPO_ROOT/examples/connections/hosts\",\"identities_root\":\"$WAYSTONE_PUBLISHD_REPO_ROOT/examples/connections/identities\"}")"
for expected in ssh-capsule production content/index.gmi; do
  case "$preview_output" in
    *"$expected"*) ;;
    *)
      echo "publishd D-Bus activation smoke: autostart PreviewPublication failed"
      echo "$preview_output"
      exit 1
      ;;
  esac
done

owner_pid="$(busctl --user call \
  org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus \
  GetConnectionUnixProcessID \
  s org.waystone.Publish1 | awk "{print \$2}")"

if [ -n "$owner_pid" ]; then
  kill -TERM "$owner_pid" 2>/dev/null || true
fi

echo "publishd D-Bus activation smoke: autostart succeeded"
'
