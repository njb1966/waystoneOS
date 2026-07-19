# WaystoneOS Checkpoint

Status: current after Publish-pane planned history preview export
Date: 2026-07-19

This checkpoint marks the current implementation state after the first repository push, the first local Workspace root configuration slice, the initial project, publish, host, identity, and audio D-Bus adapter and activation-artifact slices, the first local Workspace authoring preview slice, the Qt project creation flow, focused Qt project create/save smoke coverage, local Gemtext link validation, removable publish-target setup, Publish-pane local project previews, Publish-pane target status controls, focused Publish-pane target/status smoke coverage, Publish-pane planned history preview, Publish-pane planned history action summary, and Publish-pane planned history preview export.

## Current Position

WaystoneOS is in early OS implementation after the Phase 0 charter and architecture decision register.

The current system is a local-first development preview made of:

- Rust domain crates for project format, publish planning, host/identity metadata, audio metadata, services, publication history, and CLI output helpers
- Native CLIs for project, publish, host, identity, record, listen, and way command discovery
- D-Bus service binaries for project, publish, host, identity, and audio service boundaries
- Qt 6 C++ Workspace prototype using CLI JSON adapters and local project content editing
- Examples and invalid fixtures for project, publish, host, identity, and audio metadata behavior
- Smoke scripts for Qt startup and CLI JSON contract fields

## Workspace UI State

The Qt Workspace currently has:

- Explore pane with static placeholder data
- Explore pane active-root display for the loaded Workspace configuration
- Create pane backed by `project`, `record`, and `listen` CLI JSON output
- Create pane can create minimal projects under the configured projects root using `project create --json`
- Newly created Qt projects receive a default removable `export` target at `publish/export`
- Create pane can add removable publish targets to the selected project using `project target add-removable --json`
- Newly created projects refresh into the project list and open in the editor
- Create pane loads the selected project content index through `project inspect --json`
- Create pane provides basic Gemtext editing, saving, validation status, local preview, and local link validation
- Publish pane lists configured local projects and derives preview targets from `project inspect --json`
- Publish pane shows all discovered project targets, exposes them through a target selector, and reports preview readiness as ready, blocked, failed, no project, or no target
- Publish pane previews selected local projects through `publish --dry-run --json`, including newly created removable export targets
- Publish pane previews planned publication history records through `publish --planned-history --json`, including file-action grouping, without writing completed history
- Publish pane can save planned history previews under the selected project `history/previews/` directory through `publish --save-planned-history-preview --json`
- Operate pane backed by `host` and `identity` CLI JSON output
- Shared command execution and JSON parsing in `ui/workspace-qt/src/cli_adapter.*`
- Local root configuration in `ui/workspace-qt/src/workspace_config.*`
- Example root configuration in `ui/workspace-qt/workspace.example.ini`
- Config lookup order: explicit `--config`, user app config, repository defaults
- Explore pane editing for persistent user root settings
- `--check-roots` diagnostics for missing explicit config and missing configured roots
- Diagnostic project create/save and Publish-pane target/status smoke modes exercised by `scripts/workspace-qt-project-smoke.sh`
- Page construction in `ui/workspace-qt/src/workspace_pages.*`
- Application frame setup in `ui/workspace-qt/src/main.cpp`

The UI is intentionally local-only. It writes user root settings, creates minimal projects under the configured projects root, adds removable publish target metadata, edits selected project content index files, and saves planned history preview records under selected project `history/previews/` directories only; it does not call D-Bus, mutate remotes, unlock credentials, capture audio, or embed Browser, Helm, or Comm.

## Verification Marker

The current expected verification set is:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-smoke.sh
scripts/workspace-qt-project-smoke.sh
scripts/projectd-dbus-smoke.sh
scripts/publishd-dbus-smoke.sh
scripts/host-identity-dbus-smoke.sh
scripts/audiod-dbus-smoke.sh
scripts/projectd-dbus-activation-smoke.sh
scripts/publishd-dbus-activation-smoke.sh
scripts/host-identity-dbus-activation-smoke.sh
scripts/audiod-dbus-activation-smoke.sh
scripts/projectd-systemd-unit-smoke.sh
scripts/publishd-systemd-unit-smoke.sh
scripts/host-identity-systemd-unit-smoke.sh
scripts/audiod-systemd-unit-smoke.sh
```

Result after Publish-pane planned history preview export pass: all passed on 2026-07-19.

## Important Boundaries

- Initial repository commit and push were completed after explicit user approval.
- No files outside this repository were edited by the assistant.
- Sibling Waystone applications remain future add-ons only.
- `waystone-projectd` direct D-Bus serving is implemented for project create, list, inspect, and validate.
- `waystone-publishd` direct D-Bus serving is implemented for non-mutating publication preview and planned-history generation.
- `waystone-projectd` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-projectd` D-Bus service file and systemd user unit are present in the repo.
- `waystone-publishd` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-publishd` D-Bus service file and systemd user unit are present in the repo.
- `waystone-hostd` direct D-Bus serving is implemented for host list, inspect, and validate.
- `waystone-identityd` direct D-Bus serving is implemented for identity list, inspect, and validate.
- `waystone-audiod` direct D-Bus serving is implemented for recording metadata list, inspect, and validate.
- `waystone-hostd` and `waystone-identityd` fail cleanly without a session bus and reject duplicate bus ownership.
- `waystone-hostd` and `waystone-identityd` D-Bus service files and systemd user units are present in the repo.
- `waystone-audiod` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-audiod` D-Bus service file and systemd user unit are present in the repo.
- D-Bus autostart is verified on a private test session bus with generated temporary service files.
- Activation files have not been installed into user or system service directories.
- Remote publication execution is not implemented.
- Qt Workspace data roots default to repository examples and can be overridden with `--config` or user app config.

## Next Work Queue

Recommended next implementation step:

1. Refine Publish-pane saved-preview discoverability, such as listing saved preview records for the selected project.
2. Keep Qt Workspace on CLI adapters until D-Bus activation behavior is stable in installed environments.
3. Keep packaging/install automation deferred until the repo has a broader install layout.

Alternative next step:

- Add deeper editor diagnostics or a dedicated content-file list before expanding publish preview workflow further.
