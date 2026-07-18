# WaystoneOS Checkpoint

Status: paused for repository initialization
Date: 2026-07-18

This checkpoint marks the current implementation state before the first git commit/push.

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
- Create pane backed by `project`, `record`, and `listen` CLI JSON output
- Publish pane backed by `publish --dry-run --json`
- Operate pane backed by `host` and `identity` CLI JSON output
- Shared command execution and JSON parsing in `ui/workspace-qt/src/cli_adapter.*`
- Page construction in `ui/workspace-qt/src/workspace_pages.*`
- Application frame setup in `ui/workspace-qt/src/main.cpp`

The UI is intentionally read-only. It does not call D-Bus, mutate remotes, unlock credentials, capture audio, or embed Browser, Helm, or Comm.

## Verification Marker

The current expected verification set is:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-smoke.sh
```

Result after checkpoint documentation pass: all passed on 2026-07-18.

## Important Boundaries

- No git operations have been run by the assistant.
- No files outside this repository were edited by the assistant.
- Sibling Waystone applications remain future add-ons only.
- D-Bus daemon lifecycle is not implemented.
- Remote publication execution is not implemented.
- Qt Workspace data roots are still hardcoded to repository examples.

## Next Work Queue

Recommended next implementation step:

1. Replace hardcoded example-root assumptions in `CliAdapter` with a small local Workspace settings/config model.
2. Add configurable roots for projects, hosts, identities, and audio metadata.
3. Keep configuration file based and local-only; do not introduce D-Bus yet.
4. Add tests or smoke checks for config fallback behavior.

Alternative next step:

- Start the D-Bus adapter planning slice only after the local settings/config model is stable enough to identify what the daemons should own.
