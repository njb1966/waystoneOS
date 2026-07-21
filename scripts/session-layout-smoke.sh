#!/usr/bin/env bash
set -eu

fail() {
    echo "session-layout-smoke: $*" >&2
    exit 1
}

contains() {
    file="$1"
    expected="$2"

    case "$(cat "$file")" in
        *"$expected"*) ;;
        *) fail "$file is missing: $expected" ;;
    esac
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
desktop="$repo_root/session/waystone.desktop"
wrapper="$repo_root/session/waystone-session"
layout="$repo_root/session/install-layout.toml"

[ -f "$desktop" ] || fail "missing session desktop entry: $desktop"
[ -f "$wrapper" ] || fail "missing session wrapper: $wrapper"
[ -f "$layout" ] || fail "missing install-layout manifest: $layout"
[ -x "$wrapper" ] || fail "session wrapper is not executable: $wrapper"

contains "$desktop" "[Desktop Entry]"
contains "$desktop" "Name=WaystoneOS"
contains "$desktop" "Exec=/usr/bin/waystone-session"
contains "$desktop" "TryExec=/usr/bin/waystone-session"
contains "$desktop" "Type=Application"
contains "$desktop" "DesktopNames=WaystoneOS"

contains "$layout" 'desktop_install = "/usr/share/wayland-sessions/waystone.desktop"'
contains "$layout" 'wrapper_install = "/usr/bin/waystone-session"'
contains "$layout" 'workspace_binary_install = "/usr/bin/waystone-workspace"'
contains "$layout" 'projectd = "/usr/bin/waystone-projectd"'
contains "$layout" 'publishd = "/usr/bin/waystone-publishd"'
contains "$layout" 'hostd = "/usr/bin/waystone-hostd"'
contains "$layout" 'identityd = "/usr/bin/waystone-identityd"'
contains "$layout" 'audiod = "/usr/bin/waystone-audiod"'
contains "$layout" 'project = "/usr/share/dbus-1/services/org.waystone.Project1.service"'
contains "$layout" 'publish = "/usr/share/dbus-1/services/org.waystone.Publish1.service"'
contains "$layout" 'host = "/usr/share/dbus-1/services/org.waystone.Host1.service"'
contains "$layout" 'identity = "/usr/share/dbus-1/services/org.waystone.Identity1.service"'
contains "$layout" 'audio = "/usr/share/dbus-1/services/org.waystone.Audio1.service"'
contains "$layout" 'project = "/usr/lib/systemd/user/waystone-projectd.service"'
contains "$layout" 'publish = "/usr/lib/systemd/user/waystone-publishd.service"'
contains "$layout" 'host = "/usr/lib/systemd/user/waystone-hostd.service"'
contains "$layout" 'identity = "/usr/lib/systemd/user/waystone-identityd.service"'
contains "$layout" 'audio = "/usr/lib/systemd/user/waystone-audiod.service"'
contains "$layout" "installs_outside_repo = false"

smoke_root="$(mktemp -d /tmp/waystone-session-layout-XXXXXX)"
fake_workspace="$smoke_root/waystone-workspace"
fake_config="$repo_root/ui/workspace-qt/workspace.example.ini"

{
    printf '#!/usr/bin/env bash\n'
    printf 'set -eu\n'
    printf 'printf "fake workspace invoked\\n"\n'
    printf 'for arg in "$@"; do printf "arg:%%s\\n" "$arg"; done\n'
} > "$fake_workspace"
chmod +x "$fake_workspace"

output="$(
    WAYSTONE_WORKSPACE_BIN="$fake_workspace" \
    WAYSTONE_REPO_ROOT="$repo_root" \
    WAYSTONE_WORKSPACE_CONFIG="$fake_config" \
    "$wrapper" --check-roots 2>&1
)"

case "$output" in
    *"fake workspace invoked"*) ;;
    *) fail "wrapper did not execute fake workspace" ;;
esac

case "$output" in *"arg:--repo-root"*) ;; *) fail "wrapper did not pass --repo-root" ;; esac
case "$output" in *"arg:$repo_root"*) ;; *) fail "wrapper did not pass repo root value" ;; esac
case "$output" in *"arg:--config"*) ;; *) fail "wrapper did not pass --config" ;; esac
case "$output" in *"arg:$fake_config"*) ;; *) fail "wrapper did not pass config value" ;; esac
case "$output" in *"arg:--check-roots"*) ;; *) fail "wrapper did not pass through user arguments" ;; esac

set +e
missing_output="$(WAYSTONE_WORKSPACE_BIN="$smoke_root/missing" "$wrapper" 2>&1)"
missing_status="$?"
set -e

[ "$missing_status" -eq 127 ] || fail "missing workspace returned $missing_status instead of 127"

case "$missing_output" in
    *"workspace binary not executable"*) ;;
    *) fail "missing workspace diagnostic was not clear" ;;
esac

echo "session-layout-smoke: ok"
