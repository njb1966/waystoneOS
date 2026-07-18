# WaystoneOS Checkpoint

Status: current after Workspace root configuration
Date: 2026-07-18

This checkpoint marks the current implementation state after the first repository push and the first local Workspace root configuration slice.

## Current Position

WaystoneOS is in early OS implementation after the Phase 0 charter and architecture decision register.

The current system is a local-first, read-only development preview made of:

- Rust domain crates for project format, publish planning, host/identity metadata, audio metadata, services, publication history, and CLI output helpers
- Native CLIs for project, publish, host, identity, record, listen, and way command discovery
- Placeholder service binaries for project, host, identity, and audio service boundaries
- Qt 6 C++ Workspace scaffold using read-only CLI JSON adapters
- Examples and invalid fixtures for project, publish, host, identity, and audio metadata behavior
- Smoke scripts for Qt startup and CLI JSON contract fields

## Workspace UI State

The Qt Workspace currently has:

- Explore pane with static placeholder data
- Explore pane active-root display for the loaded Workspace configuration
- Create pane backed by `project`, `record`, and `listen` CLI JSON output
- Publish pane backed by `publish --dry-run --json`
- Operate pane backed by `host` and `identity` CLI JSON output
- Shared command execution and JSON parsing in `ui/workspace-qt/src/cli_adapter.*`
- Local root configuration in `ui/workspace-qt/src/workspace_config.*`
- Example root configuration in `ui/workspace-qt/workspace.example.ini`
- Config lookup order: explicit `--config`, user app config, repository defaults
- Explore pane editing for persistent user root settings
- `--check-roots` diagnostics for missing explicit config and missing configured roots
- Page construction in `ui/workspace-qt/src/workspace_pages.*`
- Application frame setup in `ui/workspace-qt/src/main.cpp`

The UI is intentionally local-only. It writes user root settings only; it does not call D-Bus, mutate remotes, unlock credentials, capture audio, or embed Browser, Helm, or Comm.

## Verification Marker

The current expected verification set is:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-smoke.sh
scripts/projectd-dbus-smoke.sh
```

Result after first mutating `waystone-projectd` D-Bus method pass: all passed on 2026-07-18.

## Important Boundaries

- Initial repository commit and push were completed after explicit user approval.
- No files outside this repository were edited by the assistant.
- Sibling Waystone applications remain future add-ons only.
- `waystone-projectd` direct D-Bus serving is implemented for project create, list, inspect, and validate.
- D-Bus activation and systemd user units are not implemented.
- Remote publication execution is not implemented.
- Qt Workspace data roots default to repository examples and can be overridden with `--config` or user app config.

## Next Work Queue

Recommended next implementation step:

1. Add tighter lifecycle/error tests for unavailable session bus and duplicate bus ownership.
2. Keep systemd activation deferred until direct daemon behavior stays stable.
3. Keep Qt Workspace on CLI adapters until D-Bus lifecycle and error behavior are stable.

Alternative next step:

- Keep D-Bus implementation deferred and continue extending the Qt Workspace CLI-backed prototype.
