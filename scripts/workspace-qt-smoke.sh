#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="${WAYSTONE_WORKSPACE_QT_BUILD_DIR:-/tmp/waystone-workspace-qt-build}"

cargo build \
  -p waystone-project-cli \
  -p waystone-publish-cli \
  -p waystone-host-cli \
  -p waystone-identity-cli \
  -p waystone-record-cli \
  -p waystone-listen-cli

cmake -S "$repo_root/ui/workspace-qt" -B "$build_dir"
cmake --build "$build_dir"

set +e
QT_QPA_PLATFORM=offscreen \
  timeout 5s "$build_dir/waystone-workspace" \
    --repo-root "$repo_root" \
    --no-user-config
status=$?
set -e

if [ "$status" -eq 124 ]; then
  set +e
  QT_QPA_PLATFORM=offscreen \
    timeout 5s "$build_dir/waystone-workspace" \
      --repo-root "$repo_root" \
      --config "$repo_root/ui/workspace-qt/workspace.example.ini"
  config_status=$?
  set -e
  if [ "$config_status" -ne 124 ]; then
    echo "waystone-workspace smoke: config startup failed with exit code $config_status"
    exit "$config_status"
  fi

  user_config_home="${WAYSTONE_WORKSPACE_QT_CONFIG_HOME:-/tmp/waystone-workspace-qt-config}"
  mkdir -p "$user_config_home/WaystoneOS/Waystone Workspace"
  cp "$repo_root/ui/workspace-qt/workspace.example.ini" \
    "$user_config_home/WaystoneOS/Waystone Workspace/workspace.ini"

  set +e
  XDG_CONFIG_HOME="$user_config_home" \
    QT_QPA_PLATFORM=offscreen \
    timeout 5s "$build_dir/waystone-workspace" --repo-root "$repo_root"
  user_config_status=$?
  set -e
  if [ "$user_config_status" -eq 124 ]; then
    missing_config="$repo_root/ui/workspace-qt/missing-workspace.ini"
    set +e
    missing_config_output="$(QT_QPA_PLATFORM=offscreen \
      "$build_dir/waystone-workspace" \
        --repo-root "$repo_root" \
        --config "$missing_config" \
        --check-roots 2>&1)"
    missing_config_status=$?
    set -e
    if [ "$missing_config_status" -ne 0 ]; then
      echo "waystone-workspace smoke: missing config fallback failed with exit code $missing_config_status"
      echo "$missing_config_output"
      exit "$missing_config_status"
    fi

    case "$missing_config_output" in
      *"Workspace config not found; using defaults"*) ;;
      *)
        echo "waystone-workspace smoke: missing config warning was not reported"
        echo "$missing_config_output"
        exit 1
        ;;
    esac

    bad_config_home="${WAYSTONE_WORKSPACE_QT_BAD_CONFIG_HOME:-/tmp/waystone-workspace-qt-bad-config}"
    mkdir -p "$bad_config_home"
    bad_roots_config="$bad_config_home/missing-roots.ini"
    {
      printf "[roots]\n"
      printf "projects = /tmp/waystone-workspace-missing/projects\n"
      printf "hosts = examples/connections/hosts\n"
      printf "identities = examples/connections/identities\n"
      printf "audio_metadata = examples/projects/audio-capsule.wayproject/audio/metadata\n"
    } > "$bad_roots_config"

    set +e
    missing_roots_output="$(QT_QPA_PLATFORM=offscreen \
      "$build_dir/waystone-workspace" \
        --repo-root "$repo_root" \
        --config "$bad_roots_config" \
        --check-roots 2>&1)"
    missing_roots_status=$?
    set -e
    if [ "$missing_roots_status" -ne 2 ]; then
      echo "waystone-workspace smoke: missing roots check returned $missing_roots_status"
      echo "$missing_roots_output"
      exit 1
    fi

    case "$missing_roots_output" in
      *"Configured projects root not found"*) ;;
      *)
        echo "waystone-workspace smoke: missing projects root was not reported"
        echo "$missing_roots_output"
        exit 1
        ;;
    esac

    echo "waystone-workspace smoke: default, explicit config, user config, missing config, and missing root checks succeeded"
    exit 0
  fi

  echo "waystone-workspace smoke: user config startup failed with exit code $user_config_status"
  exit "$user_config_status"
fi

if [ "$status" -eq 0 ]; then
  echo "waystone-workspace smoke: app exited before timeout"
  exit 1
fi

echo "waystone-workspace smoke: failed with exit code $status"
exit "$status"
