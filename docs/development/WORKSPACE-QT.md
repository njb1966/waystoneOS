# Waystone Workspace Qt Development

Status: scaffold
Date: 2026-07-18

The first Waystone Workspace UI source lives in:

```text
ui/workspace-qt/
```

It is a Qt 6 C++ scaffold for the operating frame defined in `docs/architecture/WORKSPACE-UI-PLAN.md` and ADR-0012.

## Current Scope

Implemented:

- Top menu bar
- Four workspace selector buttons
- Left activity navigation
- Main stacked workspace panes
- Bottom status bar
- Read-only Create-pane adapter using `project` CLI JSON output
- Read-only recording adapter using `record` and `listen` CLI JSON output
- Read-only Publish-pane adapter using `publish --dry-run --json`
- Read-only Operate-pane adapters using `host` and `identity` CLI JSON output
- `main.cpp` application frame split from page construction in `workspace_pages.cpp`
- Local Workspace root configuration in `workspace_config.cpp`
- Explore pane active-root display for the loaded Workspace configuration
- Missing root preflight errors shown in affected panes
- Static placeholder resource data for Explore only

Not implemented:

- Service calls
- D-Bus
- Remote publishing
- Audio device access
- Browser, Helm, or Comm embedding
- Custom compositor behavior

## Debian Prerequisites

Expected packages:

```text
cmake
g++
qt6-base-dev
```

Do not install packages as part of the repo build. The UI remains a native Debian prototype component with ordinary system dependencies.

The Create, Publish, and Operate panes also expect the current CLIs to be built or installed:

```bash
cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-host-cli -p waystone-identity-cli -p waystone-record-cli -p waystone-listen-cli
```

When launched from the repository root, the UI will look for matching binaries under `target/debug/` before falling back to commands on `PATH`.

## Build

```bash
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
```

The scaffold has been verified locally with Qt 6.8.2 after installing `qt6-base-dev`.

## Run

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace
```

If launched from another working directory, pass the repo root:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
```

If using explicit data roots, pass an INI file:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --config /path/to/workspace.ini
```

If no `--config` is passed, the UI checks Qt's user app config location for `workspace.ini`. Use this option to ignore user config and force repository defaults:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --no-user-config
```

Example config:

```text
ui/workspace-qt/workspace.example.ini
```

Supported keys:

```ini
[roots]
projects = examples/projects
hosts = examples/connections/hosts
identities = examples/connections/identities
audio_metadata = examples/projects/audio-capsule.wayproject/audio/metadata
```

Headless startup smoke test:

```bash
scripts/workspace-qt-smoke.sh
```

The smoke test uses Qt's `offscreen` platform and verifies default-root, explicit-config, and user-config startup.

CLI JSON contract smoke test:

```bash
scripts/cli-json-contract-smoke.sh
```

## Next Integration Boundary

The first adapter path is CLI JSON for project, recording, publish preview, host, and identity data:

- `project list --json`
- `project inspect --json`
- `project validate --json`
- `publish --dry-run --json`
- `host list --json`
- `host inspect --json`
- `host validate --json`
- `identity list --json`
- `identity inspect --json`
- `identity validate --json`
- `record list --json`
- `record inspect --json`
- `record validate --json`
- `listen library --json`

Before deeper widgets read real data, define the next adapter scope:

- C ABI/service adapter for tighter Rust crate reuse, or
- D-Bus adapter after daemon contracts exist.
