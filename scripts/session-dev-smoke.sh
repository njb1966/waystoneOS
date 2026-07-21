#!/usr/bin/env bash
set -eu

fail() {
    echo "session-dev-smoke: $*" >&2
    exit 1
}

contains() {
    text="$1"
    expected="$2"

    case "$text" in
        *"$expected"*) ;;
        *) fail "output is missing: $expected
$text" ;;
    esac
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="${WAYSTONE_WORKSPACE_QT_BUILD_DIR:-/tmp/waystone-workspace-qt-build}"
wrapper="$repo_root/session/waystone-session"
workspace="$build_dir/waystone-workspace"

[ -x "$wrapper" ] || fail "session wrapper is not executable: $wrapper"

cmake -S "$repo_root/ui/workspace-qt" -B "$build_dir"
cmake --build "$build_dir"

[ -x "$workspace" ] || fail "Workspace binary is not executable after build: $workspace"

default_output="$(
    QT_QPA_PLATFORM=offscreen \
    WAYSTONE_WORKSPACE_BIN="$workspace" \
    WAYSTONE_REPO_ROOT="$repo_root" \
    "$wrapper" --no-user-config --check-roots 2>&1
)"

contains "$default_output" "config source: defaults"
contains "$default_output" "roots: ok"

smoke_root="$(mktemp -d /tmp/waystone-session-dev-smoke-XXXXXX)"
projects_root="$smoke_root/projects"
hosts_root="$smoke_root/hosts"
identities_root="$smoke_root/identities"
audio_root="$smoke_root/audio"
good_config="$smoke_root/workspace.ini"
bad_config="$smoke_root/missing-roots.ini"

mkdir -p "$projects_root" "$hosts_root" "$identities_root" "$audio_root"

{
    printf "[roots]\n"
    printf "projects = %s\n" "$projects_root"
    printf "hosts = %s\n" "$hosts_root"
    printf "identities = %s\n" "$identities_root"
    printf "audio_metadata = %s\n" "$audio_root"
} > "$good_config"

explicit_output="$(
    QT_QPA_PLATFORM=offscreen \
    WAYSTONE_WORKSPACE_BIN="$workspace" \
    WAYSTONE_REPO_ROOT="$repo_root" \
    WAYSTONE_WORKSPACE_CONFIG="$good_config" \
    "$wrapper" --check-roots 2>&1
)"

contains "$explicit_output" "config source: explicit"
contains "$explicit_output" "config path: $good_config"
contains "$explicit_output" "roots: ok"

{
    printf "[roots]\n"
    printf "projects = %s\n" "$smoke_root/missing/projects"
    printf "hosts = %s\n" "$hosts_root"
    printf "identities = %s\n" "$identities_root"
    printf "audio_metadata = %s\n" "$audio_root"
} > "$bad_config"

set +e
missing_output="$(
    QT_QPA_PLATFORM=offscreen \
    WAYSTONE_WORKSPACE_BIN="$workspace" \
    WAYSTONE_REPO_ROOT="$repo_root" \
    WAYSTONE_WORKSPACE_CONFIG="$bad_config" \
    "$wrapper" --check-roots 2>&1
)"
missing_status="$?"
set -e

[ "$missing_status" -eq 2 ] || fail "missing-root check returned $missing_status instead of 2"
contains "$missing_output" "config source: explicit"
contains "$missing_output" "config path: $bad_config"
contains "$missing_output" "Configured projects root not found: $smoke_root/missing/projects"

echo "session-dev-smoke: ok"
