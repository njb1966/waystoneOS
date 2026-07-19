# WaystoneOS Checkpoint

Status: current after Qt feed-entry preparation controls
Date: 2026-07-19

This checkpoint marks the current implementation state after the first repository push, the first local Workspace root configuration slice, the initial project, publish, host, identity, and audio D-Bus adapter and activation-artifact slices, the first local Workspace authoring preview slice, the Qt project creation flow, focused Qt project create/save smoke coverage, local Gemtext link validation, removable publish-target setup, Create-pane content file listing, Create-pane content file filtering, Create-pane content file detail, Publish-pane local project previews, Publish-pane target status controls, focused Publish-pane target/status smoke coverage, Publish-pane planned history preview, Publish-pane planned history action summary, Publish-pane planned history preview export, Publish-pane saved preview listing, Publish-pane saved preview detail loading, Publish-pane saved preview selection preservation, Publish-pane saved preview comparison aid, Publish-pane saved preview filtering, Publish-pane target overview, Publish-pane target overview selection, Publish-pane project filtering, the Phase 0/0.1 alignment audit, the local audio attachment slice, Create-pane recording attachment controls, audio-capable project creation defaults, feed-entry metadata preparation, audio publication handoff validation, and Qt feed-entry preparation controls.

## Current Position

WaystoneOS is in early OS implementation after the Phase 0 charter and architecture decision register.

The current alignment marker is [PHASE-0-0.1-ALIGNMENT.md](PHASE-0-0.1-ALIGNMENT.md).

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
- Create pane can create projects under the configured projects root using `project create --json`
- `audio-series` and `mixed-publication` projects receive audio/feed scaffold defaults from the project creation contract
- Newly created Qt projects receive a default removable `export` target at `publish/export`
- Create pane can add removable publish targets to the selected project using `project target add-removable --json`
- Newly created projects refresh into the project list and open in the editor
- Create pane loads the selected project content index through `project inspect --json`
- Create pane provides basic Gemtext editing, saving, validation status, local preview, and local link validation
- Create pane lists files under the selected project's content root with relative path, size, and full path
- Create pane filters visible content-root files by relative path or full path without changing the editable content index binding
- Create pane shows read-only detail for the selected content-root file, including whether it is the editable content index
- Create pane can attach an existing project-local audio master/publication copy through `record attach --json` when the selected project has `[audio].metadata` configured
- Create pane can prepare feed-entry sidecars and show publication/feed-entry validation status through `record prepare-feed-entry --json`, `record validate-publication --json`, and `record validate-feed-entry --json`
- Publish pane lists configured local projects and derives preview targets from `project inspect --json`
- Publish pane shows all discovered project targets, exposes them through a target selector, and reports preview readiness as ready, blocked, failed, no project, or no target
- Publish pane previews selected local projects through `publish --dry-run --json`, including newly created removable export targets
- Publish pane previews planned publication history records through `publish --planned-history --json`, including file-action grouping, without writing completed history
- Publish pane can save planned history previews under the selected project `history/previews/` directory through `publish --save-planned-history-preview --json`
- Publish pane lists saved planned history previews for the selected project through `publish --list-planned-history-previews --json`
- Publish pane loads selected saved preview TOML through `publish --read-planned-history-preview --json`
- Publish pane preserves the selected saved preview row across preview-list refreshes when that row still exists
- Publish pane compares generated planned history against the selected saved preview and reports the first differing line
- Publish pane filters saved previews by filename or path without reading outside the selected project
- Publish pane shows a compact per-target overview for the selected project using read-only dry-run status, method, upload count, verification count, and destination
- Publish pane lets target overview row selection choose the active target and refresh the existing preview/history panes
- Publish pane filters visible projects by project name, ID, type, path, or target names
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

The UI is intentionally local-only. It writes user root settings, creates projects under the configured projects root, adds removable publish target metadata, edits selected project content index files, saves planned history preview records under selected project `history/previews/` directories, lists those saved preview records, reads selected preview TOML only from that project-local preview directory, creates selected project audio metadata sidecars, and creates selected project feed-entry sidecars; it does not call D-Bus, mutate remotes, unlock credentials, capture audio, generate feed XML, or embed Browser, Helm, or Comm.

The Create-pane audio attachment controls call `record attach --json`. They
create metadata sidecars for existing project-local master and publication-copy
files and record feed enclosure handoff fields. They do not copy audio,
transcode, generate feeds, overwrite existing sidecars, call D-Bus, or access
audio devices.

The Create-pane feed-entry controls call `record prepare-feed-entry --json`,
`record validate-publication --json`, and `record validate-feed-entry --json`.
`record prepare-feed-entry --json` creates a project-local
`feeds/entries/<recording-id>.toml` sidecar from an existing recording sidecar
and published audio reference. This is a create-only metadata preparation
contract. It does not generate or update feed XML.

The `record validate-publication --json` and `record validate-feed-entry
--json` commands validate local publication-copy and feed-entry handoff metadata
in project context. They check required fields, project-relative paths,
referenced local files, feed-entry consistency with recording metadata, and
duplicate feed-entry IDs. They do not modify files.

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

Result after Qt feed-entry preparation controls: all relevant checks passed on
2026-07-19.

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
- `record attach` creates local audio metadata sidecars under a project's configured `[audio].metadata` root without copying audio, transcoding, generating feeds, or overwriting existing sidecars.
- `record prepare-feed-entry` creates local feed-entry metadata sidecars under `feeds/entries/` without generating or updating feed XML.
- `record validate-publication` and `record validate-feed-entry` validate local audio publication handoff metadata without mutating files.
- The Qt Create pane exposes `record attach` through local CLI adapters for selected projects with audio metadata configured.
- The Qt Create pane exposes feed-entry preparation and publication/feed-entry validation status through local CLI adapters.
- `project create` scaffolds audio/feed defaults for `audio-series` and `mixed-publication` projects.
- `waystone-hostd` and `waystone-identityd` fail cleanly without a session bus and reject duplicate bus ownership.
- `waystone-hostd` and `waystone-identityd` D-Bus service files and systemd user units are present in the repo.
- `waystone-audiod` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-audiod` D-Bus service file and systemd user unit are present in the repo.
- D-Bus autostart is verified on a private test session bus with generated temporary service files.
- Activation files have not been installed into user or system service directories.
- `waystone-audiod` remains read-only over D-Bus; the new attachment, feed-entry preparation, and project-context publication validation operations are not exposed through IPC yet.
- Remote publication execution is not implemented.
- Qt Workspace data roots default to repository examples and can be overridden with `--config` or user app config.

## Next Work Queue

Recommended next implementation step:

1. Start a minimal feed XML generator from validated `feeds/entries/` sidecars.
2. Keep Qt Workspace on CLI adapters until D-Bus activation behavior is stable in installed environments.
3. Keep packaging/install automation deferred until the repo has a broader install layout.

Alternative next step:

- Add additional Create-pane error-detail rendering for failed publication/feed-entry validation.
