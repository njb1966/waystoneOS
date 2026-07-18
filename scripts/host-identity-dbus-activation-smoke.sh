#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
activation_root="${WAYSTONE_HOST_IDENTITY_DBUS_ACTIVATION_ROOT:-/tmp/waystone-host-identity-dbus-activation-$$}"
service_dir="$activation_root/dbus-1/services"
host_service_file="$service_dir/org.waystone.Host1.service"
identity_service_file="$service_dir/org.waystone.Identity1.service"
hostd_path="$repo_root/target/debug/waystone-hostd"
identityd_path="$repo_root/target/debug/waystone-identityd"

cd "$repo_root"

cargo build -p waystone-hostd -p waystone-identityd

mkdir -p "$service_dir"

{
  printf "[D-BUS Service]\n"
  printf "Name=org.waystone.Host1\n"
  printf "Exec=%s\n" "$hostd_path"
} > "$host_service_file"

{
  printf "[D-BUS Service]\n"
  printf "Name=org.waystone.Identity1\n"
  printf "Exec=%s\n" "$identityd_path"
} > "$identity_service_file"

export WAYSTONE_HOST_IDENTITY_REPO_ROOT="$repo_root"

XDG_DATA_HOME="$activation_root" dbus-run-session -- bash -c '
set -eu

host_list_output="$(busctl --user call \
  org.waystone.Host1 \
  /org/waystone/Host \
  org.waystone.Host1 \
  ListHosts \
  s "{\"root\":\"$WAYSTONE_HOST_IDENTITY_REPO_ROOT/examples/connections/hosts\"}")"
case "$host_list_output" in
  *offgridholdout.org*) ;;
  *)
    echo "host/identity D-Bus activation smoke: autostart ListHosts failed"
    echo "$host_list_output"
    exit 1
    ;;
esac

identity_list_output="$(busctl --user call \
  org.waystone.Identity1 \
  /org/waystone/Identity \
  org.waystone.Identity1 \
  ListIdentities \
  s "{\"root\":\"$WAYSTONE_HOST_IDENTITY_REPO_ROOT/examples/connections/identities\"}")"
case "$identity_list_output" in
  *nick-pub*) ;;
  *)
    echo "host/identity D-Bus activation smoke: autostart ListIdentities failed"
    echo "$identity_list_output"
    exit 1
    ;;
esac

stop_owner() {
  bus_name="$1"
  owner_pid="$(busctl --user call \
    org.freedesktop.DBus \
    /org/freedesktop/DBus \
    org.freedesktop.DBus \
    GetConnectionUnixProcessID \
    s "$bus_name" | awk "{print \$2}")"

  if [ -n "$owner_pid" ]; then
    kill -TERM "$owner_pid" 2>/dev/null || true
  fi
}

stop_owner org.waystone.Host1
stop_owner org.waystone.Identity1

echo "host/identity D-Bus activation smoke: autostart succeeded"
'
