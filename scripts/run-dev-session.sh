#!/usr/bin/env bash
set -eu

usage() {
    cat <<'USAGE'
Usage: scripts/run-dev-session.sh [OPTIONS] [-- WORKSPACE_ARGS...]

Build and launch the repo-local WaystoneOS development preview through
session/waystone-session without installing files outside this repository.

Options:
  --config PATH       Pass an explicit Workspace config file
  --no-user-config    Ignore the user's Qt app config and use repo defaults
  --check-roots       Validate configured roots and exit without opening the UI
  --no-build          Reuse an existing Qt build under the build directory
  --help              Show this help

Environment:
  WAYSTONE_WORKSPACE_QT_BUILD_DIR  Qt build directory
                                  default: /tmp/waystone-workspace-qt-build
USAGE
}

fail() {
    echo "run-dev-session: $*" >&2
    exit 1
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="${WAYSTONE_WORKSPACE_QT_BUILD_DIR:-/tmp/waystone-workspace-qt-build}"
wrapper="$repo_root/session/waystone-session"
workspace="$build_dir/waystone-workspace"
config_path=""
build=1
workspace_args=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --config)
            [ "$#" -ge 2 ] || fail "--config requires a path"
            config_path="$2"
            shift 2
            ;;
        --no-user-config)
            workspace_args+=("--no-user-config")
            shift
            ;;
        --check-roots)
            workspace_args+=("--check-roots")
            shift
            ;;
        --no-build)
            build=0
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        --)
            shift
            while [ "$#" -gt 0 ]; do
                workspace_args+=("$1")
                shift
            done
            ;;
        *)
            workspace_args+=("$1")
            shift
            ;;
    esac
done

[ -x "$wrapper" ] || fail "session wrapper is not executable: $wrapper"

if [ "$build" -eq 1 ]; then
    cargo build \
        -p waystone-project-cli \
        -p waystone-publish-cli \
        -p waystone-host-cli \
        -p waystone-identity-cli \
        -p waystone-record-cli \
        -p waystone-listen-cli

    cmake -S "$repo_root/ui/workspace-qt" -B "$build_dir"
    cmake --build "$build_dir"
fi

[ -x "$workspace" ] || fail "Workspace binary is not executable: $workspace"

if [ "$config_path" != "" ]; then
    export WAYSTONE_WORKSPACE_CONFIG="$config_path"
fi

export WAYSTONE_WORKSPACE_BIN="$workspace"
export WAYSTONE_REPO_ROOT="$repo_root"

exec "$wrapper" "${workspace_args[@]}"
