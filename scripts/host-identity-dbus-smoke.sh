#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
hostd_log="${WAYSTONE_HOSTD_DBUS_LOG:-/tmp/waystone-hostd-dbus-smoke.log}"
identityd_log="${WAYSTONE_IDENTITYD_DBUS_LOG:-/tmp/waystone-identityd-dbus-smoke.log}"
cd "$repo_root"

cargo build -p waystone-hostd -p waystone-identityd

check_missing_bus() {
  daemon_name="$1"
  bus_suffix="$2"
  missing_bus="unix:path=/tmp/waystone-${bus_suffix}-missing-bus-$$"

  set +e
  missing_bus_output="$(DBUS_SESSION_BUS_ADDRESS="$missing_bus" \
    "target/debug/${daemon_name}" 2>&1)"
  missing_bus_status=$?
  set -e

  if [ "$missing_bus_status" -eq 0 ]; then
    echo "${daemon_name} D-Bus smoke: daemon started without an available session bus"
    echo "$missing_bus_output"
    exit 1
  fi

  case "$missing_bus_output" in
    *"${daemon_name}:"*) ;;
    *)
      echo "${daemon_name} D-Bus smoke: unavailable bus failure was not reported clearly"
      echo "$missing_bus_output"
      exit 1
      ;;
  esac
}

check_missing_bus waystone-hostd hostd
check_missing_bus waystone-identityd identityd

dbus-run-session -- bash -c '
set -eu

target/debug/waystone-hostd > "$0" 2>&1 &
hostd_pid=$!
target/debug/waystone-identityd > "$1" 2>&1 &
identityd_pid=$!

cleanup() {
  kill -TERM "$hostd_pid" 2>/dev/null || true
  kill -TERM "$identityd_pid" 2>/dev/null || true
  wait "$hostd_pid" 2>/dev/null || true
  wait "$identityd_pid" 2>/dev/null || true
}
trap cleanup EXIT

wait_for_name() {
  bus_name="$1"
  log_path="$2"
  ready=0

  for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    if busctl --user status "$bus_name" >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.1
  done

  if [ "$ready" -ne 1 ]; then
    echo "host/identity D-Bus smoke: daemon did not claim $bus_name"
    cat "$log_path"
    exit 1
  fi
}

check_duplicate_fails() {
  daemon_name="$1"
  log_path="$2.duplicate"

  set +e
  timeout 3s "target/debug/${daemon_name}" > "$log_path" 2>&1
  duplicate_status=$?
  set -e

  if [ "$duplicate_status" -eq 0 ] || [ "$duplicate_status" -eq 124 ]; then
    echo "${daemon_name} D-Bus smoke: duplicate daemon did not fail quickly"
    cat "$log_path"
    exit 1
  fi

  case "$(cat "$log_path")" in
    *"${daemon_name}:"*) ;;
    *)
      echo "${daemon_name} D-Bus smoke: duplicate ownership failure was not reported clearly"
      cat "$log_path"
      exit 1
      ;;
  esac
}

wait_for_name org.waystone.Host1 "$0"
wait_for_name org.waystone.Identity1 "$1"
check_duplicate_fails waystone-hostd "$0"
check_duplicate_fails waystone-identityd "$1"

host_list_output="$(busctl --user call \
  org.waystone.Host1 \
  /org/waystone/Host \
  org.waystone.Host1 \
  ListHosts \
  s "{\"root\":\"examples/connections/hosts\"}")"
case "$host_list_output" in
  *offgridholdout.org*) ;;
  *)
    echo "host D-Bus smoke: ListHosts did not report expected host"
    echo "$host_list_output"
    exit 1
    ;;
esac

host_inspect_output="$(busctl --user call \
  org.waystone.Host1 \
  /org/waystone/Host \
  org.waystone.Host1 \
  InspectHost \
  s "{\"path\":\"examples/connections/hosts/offgridholdout.toml\"}")"
case "$host_inspect_output" in
  *offgridholdout.org*) ;;
  *)
    echo "host D-Bus smoke: InspectHost did not report expected host"
    echo "$host_inspect_output"
    exit 1
    ;;
esac

host_validate_output="$(busctl --user call \
  org.waystone.Host1 \
  /org/waystone/Host \
  org.waystone.Host1 \
  ValidateHost \
  s "{\"path\":\"tests/fixtures/hosts/invalid-trust/host.toml\"}")"
case "$host_validate_output" in
  *unsupported_trust_state*) ;;
  *)
    echo "host D-Bus smoke: ValidateHost did not report invalid fixture"
    echo "$host_validate_output"
    exit 1
    ;;
esac

host_bad_request_output="$(busctl --user call \
  org.waystone.Host1 \
  /org/waystone/Host \
  org.waystone.Host1 \
  ListHosts \
  s "{bad-json")"
case "$host_bad_request_output" in
  *invalid_request*) ;;
  *)
    echo "host D-Bus smoke: invalid request was not reported"
    echo "$host_bad_request_output"
    exit 1
    ;;
esac

identity_list_output="$(busctl --user call \
  org.waystone.Identity1 \
  /org/waystone/Identity \
  org.waystone.Identity1 \
  ListIdentities \
  s "{\"root\":\"examples/connections/identities\"}")"
case "$identity_list_output" in
  *nick-pub*) ;;
  *)
    echo "identity D-Bus smoke: ListIdentities did not report expected identity"
    echo "$identity_list_output"
    exit 1
    ;;
esac

identity_inspect_output="$(busctl --user call \
  org.waystone.Identity1 \
  /org/waystone/Identity \
  org.waystone.Identity1 \
  InspectIdentity \
  s "{\"path\":\"examples/connections/identities/nick-pub.toml\"}")"
case "$identity_inspect_output" in
  *nick-pub*) ;;
  *)
    echo "identity D-Bus smoke: InspectIdentity did not report expected identity"
    echo "$identity_inspect_output"
    exit 1
    ;;
esac

identity_validate_output="$(busctl --user call \
  org.waystone.Identity1 \
  /org/waystone/Identity \
  org.waystone.Identity1 \
  ValidateIdentity \
  s "{\"path\":\"tests/fixtures/identities/private-key-leak/identity.toml\"}")"
case "$identity_validate_output" in
  *private_key_material_present*) ;;
  *)
    echo "identity D-Bus smoke: ValidateIdentity did not report invalid fixture"
    echo "$identity_validate_output"
    exit 1
    ;;
esac

identity_bad_request_output="$(busctl --user call \
  org.waystone.Identity1 \
  /org/waystone/Identity \
  org.waystone.Identity1 \
  ListIdentities \
  s "{bad-json")"
case "$identity_bad_request_output" in
  *invalid_request*) ;;
  *)
    echo "identity D-Bus smoke: invalid request was not reported"
    echo "$identity_bad_request_output"
    exit 1
    ;;
esac

echo "host/identity D-Bus smoke: adapters succeeded"
' "$hostd_log" "$identityd_log"
