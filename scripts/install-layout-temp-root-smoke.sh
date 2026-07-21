#!/usr/bin/env bash
set -eu

fail() {
    echo "install-layout-temp-root-smoke: $*" >&2
    exit 1
}

contains_file() {
    file="$1"
    expected="$2"

    case "$(cat "$file")" in
        *"$expected"*) ;;
        *) fail "$file is missing: $expected" ;;
    esac
}

stage_file() {
    source_path="$1"
    install_path="$2"
    mode="$3"
    staged_path="$install_root${install_path}"

    [ -f "$source_path" ] || fail "missing source artifact: $source_path"
    mkdir -p "$(dirname "$staged_path")"
    cp "$source_path" "$staged_path"
    chmod "$mode" "$staged_path"
}

stage_fake_binary() {
    install_path="$1"
    staged_path="$install_root${install_path}"

    mkdir -p "$(dirname "$staged_path")"
    {
        printf '#!/usr/bin/env sh\n'
        printf 'exit 0\n'
    } > "$staged_path"
    chmod 755 "$staged_path"
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
layout="$repo_root/session/install-layout.toml"
install_root="${WAYSTONE_INSTALL_LAYOUT_SMOKE_ROOT:-$(mktemp -d /tmp/waystone-install-layout-XXXXXX)}"

[ -f "$layout" ] || fail "missing install layout manifest: $layout"

contains_file "$layout" 'desktop_source = "session/waystone.desktop"'
contains_file "$layout" 'desktop_install = "/usr/share/wayland-sessions/waystone.desktop"'
contains_file "$layout" 'wrapper_source = "session/waystone-session"'
contains_file "$layout" 'wrapper_install = "/usr/bin/waystone-session"'
contains_file "$layout" 'workspace_binary_install = "/usr/bin/waystone-workspace"'
contains_file "$layout" 'installs_outside_repo = false'
contains_file "$layout" 'dbus_activation_install = false'

stage_file "$repo_root/session/waystone.desktop" \
    "/usr/share/wayland-sessions/waystone.desktop" 644
stage_file "$repo_root/session/waystone-session" "/usr/bin/waystone-session" 755

stage_fake_binary "/usr/bin/waystone-workspace"
stage_fake_binary "/usr/bin/waystone-projectd"
stage_fake_binary "/usr/bin/waystone-publishd"
stage_fake_binary "/usr/bin/waystone-hostd"
stage_fake_binary "/usr/bin/waystone-identityd"
stage_fake_binary "/usr/bin/waystone-audiod"

stage_file "$repo_root/services/projectd/dbus/org.waystone.Project1.service" \
    "/usr/share/dbus-1/services/org.waystone.Project1.service" 644
stage_file "$repo_root/services/publishd/dbus/org.waystone.Publish1.service" \
    "/usr/share/dbus-1/services/org.waystone.Publish1.service" 644
stage_file "$repo_root/services/hostd/dbus/org.waystone.Host1.service" \
    "/usr/share/dbus-1/services/org.waystone.Host1.service" 644
stage_file "$repo_root/services/identityd/dbus/org.waystone.Identity1.service" \
    "/usr/share/dbus-1/services/org.waystone.Identity1.service" 644
stage_file "$repo_root/services/audiod/dbus/org.waystone.Audio1.service" \
    "/usr/share/dbus-1/services/org.waystone.Audio1.service" 644

stage_file "$repo_root/services/projectd/systemd/waystone-projectd.service" \
    "/usr/lib/systemd/user/waystone-projectd.service" 644
stage_file "$repo_root/services/publishd/systemd/waystone-publishd.service" \
    "/usr/lib/systemd/user/waystone-publishd.service" 644
stage_file "$repo_root/services/hostd/systemd/waystone-hostd.service" \
    "/usr/lib/systemd/user/waystone-hostd.service" 644
stage_file "$repo_root/services/identityd/systemd/waystone-identityd.service" \
    "/usr/lib/systemd/user/waystone-identityd.service" 644
stage_file "$repo_root/services/audiod/systemd/waystone-audiod.service" \
    "/usr/lib/systemd/user/waystone-audiod.service" 644

desktop="$install_root/usr/share/wayland-sessions/waystone.desktop"
wrapper="$install_root/usr/bin/waystone-session"

contains_file "$desktop" "Exec=/usr/bin/waystone-session"
contains_file "$desktop" "TryExec=/usr/bin/waystone-session"
[ -x "$wrapper" ] || fail "staged wrapper is not executable"

check_service() {
    service_name="$1"
    bus_name="$2"
    binary_name="$3"
    unit_name="$4"

    service_file="$install_root/usr/share/dbus-1/services/$service_name"
    unit_file="$install_root/usr/lib/systemd/user/$unit_name"
    binary_file="$install_root/usr/bin/$binary_name"

    [ -f "$service_file" ] || fail "missing staged D-Bus service: $service_file"
    [ -f "$unit_file" ] || fail "missing staged systemd user unit: $unit_file"
    [ -x "$binary_file" ] || fail "missing staged binary placeholder: $binary_file"

    contains_file "$layout" "\"/usr/bin/$binary_name\""
    contains_file "$layout" "\"/usr/share/dbus-1/services/$service_name\""
    contains_file "$layout" "\"/usr/lib/systemd/user/$unit_name\""
    contains_file "$service_file" "Name=$bus_name"
    contains_file "$service_file" "Exec=/usr/bin/$binary_name"
    contains_file "$service_file" "SystemdService=$unit_name"
    contains_file "$unit_file" "ExecStart=/usr/bin/$binary_name"

    if command -v systemd-analyze >/dev/null 2>&1; then
        verify_unit="$install_root/verify/$unit_name"
        mkdir -p "$(dirname "$verify_unit")"
        sed "s#ExecStart=/usr/bin/$binary_name#ExecStart=$binary_file#" \
            "$unit_file" > "$verify_unit"
        systemd-analyze verify "$verify_unit" >/dev/null
    fi
}

check_service "org.waystone.Project1.service" "org.waystone.Project1" \
    "waystone-projectd" "waystone-projectd.service"
check_service "org.waystone.Publish1.service" "org.waystone.Publish1" \
    "waystone-publishd" "waystone-publishd.service"
check_service "org.waystone.Host1.service" "org.waystone.Host1" \
    "waystone-hostd" "waystone-hostd.service"
check_service "org.waystone.Identity1.service" "org.waystone.Identity1" \
    "waystone-identityd" "waystone-identityd.service"
check_service "org.waystone.Audio1.service" "org.waystone.Audio1" \
    "waystone-audiod" "waystone-audiod.service"

wrapper_output="$(
    WAYSTONE_WORKSPACE_BIN="$install_root/usr/bin/waystone-workspace" \
    "$wrapper" 2>&1
)"

case "$wrapper_output" in
    "") ;;
    *) fail "staged wrapper fake Workspace produced unexpected output: $wrapper_output" ;;
esac

echo "install-layout-temp-root-smoke: ok ($install_root)"
