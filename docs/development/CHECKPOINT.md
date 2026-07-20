# WaystoneOS Checkpoint

Status: current after Qt completed-history read-only display
Date: 2026-07-20

This checkpoint marks the current implementation state after the first repository push, the first local Workspace root configuration slice, the initial project, publish, host, identity, and audio D-Bus adapter and activation-artifact slices, the first local Workspace authoring preview slice, the Qt project creation flow, focused Qt project create/save smoke coverage, local Gemtext link validation, removable publish-target setup, Create-pane content file listing, Create-pane content file filtering, Create-pane content file detail, Publish-pane local project previews, Publish-pane target status controls, focused Publish-pane target/status smoke coverage, Publish-pane planned history preview, Publish-pane planned history action summary, Publish-pane planned history preview export, Publish-pane saved preview listing, Publish-pane saved preview detail loading, Publish-pane saved preview selection preservation, Publish-pane saved preview comparison aid, Publish-pane saved preview filtering, Publish-pane target overview, Publish-pane target overview selection, Publish-pane project filtering, the Phase 0/0.1 alignment audit, the local audio attachment slice, Create-pane recording attachment controls, audio-capable project creation defaults, feed-entry metadata preparation, audio publication handoff validation, Qt feed-entry preparation controls, minimal feed XML generation and local Atom feed merge/update, Qt feed generation controls, Publish-pane feed readiness reporting, real `ffmpeg/libopus` Opus publication-copy export, Qt Create-pane controls for that export command, Publish-pane invalid feed-entry diagnostics, Publish-pane validation detail for selected feed-entry diagnostics, the CLI/service recording metadata update command, Qt Create-pane controls for that update command, the CLI/service feed-entry update command, Qt Create-pane controls for that feed-entry update command, Publish-to-Create handoff for selected invalid feed-entry diagnostics, narrow local WAV master capture from explicit `ffmpeg` input sources, Qt Create-pane controls for that capture command, and `waystone-audiod` D-Bus methods for the existing local audio/feed service operations.
It also includes local completed publication-history result records through the
publish CLI, publish service crate, and `waystone-publishd` D-Bus adapter.
It also includes a non-mutating publication readiness validation report through
the publish CLI, publish service crate, and `waystone-publishd` D-Bus adapter.
The Qt Publish pane now surfaces that report and read-only completed-history
record list/detail views through the local CLI adapter.

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
- Create pane can create an encoded Opus publication copy from an existing project-local master through `record export-opus --json`
- Create pane can create a WAV master from an explicit `ffmpeg` input source through `record capture --json`
- Create pane can attach an existing project-local audio master/publication copy through `record attach --json` when the selected project has `[audio].metadata` configured
- Create pane can update existing recording metadata sidecars through `record update --json`
- Create pane can prepare feed-entry sidecars and show publication/feed-entry validation status through `record prepare-feed-entry --json`, `record validate-publication --json`, and `record validate-feed-entry --json`
- Create pane can refresh existing feed-entry sidecars from current recording metadata through `record update-feed-entry --json`
- Publish pane lists configured local projects and derives preview targets from `project inspect --json`
- Publish pane shows all discovered project targets, exposes them through a target selector, and reports preview readiness as ready, blocked, failed, no project, or no target
- Publish pane previews selected local projects through `publish --dry-run --json`, including newly created removable export targets
- Publish pane shows publication readiness validation through `publish --validate --json`
- Publish pane reports configured feed readiness from dry-run output, including feed path, feed XML existence, prepared entry count, invalid entry count, and per-invalid-sidecar diagnostic paths plus issue text
- Publish pane can run `record validate-feed-entry --json` for a selected invalid feed-entry diagnostic and show the full validation detail
- Publish pane can open a selected invalid feed-entry diagnostic in the Create pane with the matching project and derived recording ID loaded
- Publish pane previews planned publication history records through `publish --planned-history --json`, including file-action grouping, without writing completed history
- Publish pane can save planned history previews under the selected project `history/previews/` directory through `publish --save-planned-history-preview --json`
- Publish pane lists saved planned history previews for the selected project through `publish --list-planned-history-previews --json`
- Publish pane loads selected saved preview TOML through `publish --read-planned-history-preview --json`
- Publish pane preserves the selected saved preview row across preview-list refreshes when that row still exists
- Publish pane compares generated planned history against the selected saved preview and reports the first differing line
- Publish pane filters saved previews by filename or path without reading outside the selected project
- Publish pane lists saved completed publication-history records for the selected project through `publish --list-completed-history --json`
- Publish pane loads selected completed publication-history record TOML through `publish --read-completed-history --json`
- Publish pane filters saved completed records by filename or path without reading outside the selected project
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

The UI is intentionally local-only. It writes user root settings, creates projects under the configured projects root, adds removable publish target metadata, edits selected project content index files, saves planned history preview records under selected project `history/previews/` directories, lists those saved preview records, reads selected preview TOML only from that project-local preview directory, creates selected project WAV masters and Opus publication copies through explicit `ffmpeg` commands, creates and updates selected project audio metadata sidecars, creates and updates selected project feed-entry sidecars, and generates the selected project's feed XML from prepared sidecars; it does not call D-Bus, mutate remotes, unlock credentials, enumerate audio devices, or embed Browser, Helm, or Comm.

The Create-pane audio attachment controls call `record attach --json`. They
create metadata sidecars for existing project-local master and publication-copy
files and record feed enclosure handoff fields. They do not copy audio,
transcode, generate feeds, overwrite existing sidecars, call D-Bus, or access
audio devices.

The `record update --json` command rewrites an existing recording sidecar under
the selected project's configured `[audio].metadata` root. It preserves the
sidecar path, embedded `recording.id`, and optional measurement fields while
replacing title, master, publication-copy, feed, entry ID, and MIME fields. It
requires replacement master and publication-copy files to exist inside the
project. It does not edit audio, create new sidecars, update prepared
feed-entry sidecars, merge feed XML, or call D-Bus. The Qt Create pane exposes
it through the local CLI adapter.

The Create-pane export control calls `record export-opus --json`. It creates
an encoded `.opus` publication copy from an existing project-local master file
through `ffmpeg/libopus`, refuses to overwrite an existing output, and reports
`engine = "ffmpeg"`. It does not capture audio, edit metadata sidecars, publish
remotely, call D-Bus, or access audio devices.

The `record capture --json` command writes a WAV master under the selected
project's configured `[audio].masters` root from an explicit `ffmpeg` input
format and input source. It records for a bounded duration, writes mono 48 kHz
PCM WAV through a temporary file, refuses to overwrite existing masters, and
reports the capture engine plus measurements in JSON output. It does not
enumerate devices, attach metadata, export publication copies, generate feeds,
publish remotely, or call D-Bus. The Qt Create pane exposes it through the
local CLI adapter with explicit input format, input source, and duration
fields.

The Create-pane feed-entry controls call `record prepare-feed-entry --json`,
`record validate-publication --json`, and `record validate-feed-entry --json`.
`record prepare-feed-entry --json` creates a project-local
`feeds/entries/<recording-id>.toml` sidecar from an existing recording sidecar
and published audio reference. This is a create-only metadata preparation
contract. It does not generate or update feed XML.

The `record update-feed-entry --json` command refreshes an existing
`feeds/entries/<recording-id>.toml` sidecar from the current recording
sidecar. It updates entry/enclosure handoff fields, `updated`, and `summary`,
but it does not create missing feed-entry sidecars, generate feed XML, merge
existing feed XML, publish remotely, or call D-Bus. The Qt Create pane exposes
it through the local CLI adapter.

The `record generate-feed --json` command is implemented as a minimal local
Atom generator. It reads enabled `[feed]` project configuration, validates every
`feeds/entries/*.toml` sidecar, sorts sidecar-managed entries by descending
update time, and atomically writes the configured feed XML file. If the
configured feed already contains Atom entries, entries with IDs matching
prepared sidecars are replaced from sidecar metadata and unrelated existing
entries are preserved in their current XML form. It does not support non-Atom
feeds, copy audio, transcode audio, merge remote feed state, publish remotely,
or expose the operation through D-Bus yet. The Qt Create pane exposes it
through the local CLI adapter.

The `record validate-publication --json` and `record validate-feed-entry
--json` commands validate local publication-copy and feed-entry handoff metadata
in project context. They check required fields, project-relative paths,
referenced local files, feed-entry consistency with recording metadata, and
duplicate feed-entry IDs. They do not modify files.

The `publish --dry-run --json` command now includes a `feed` object for every
preview. It reports whether a feed is configured and enabled, the feed type and
path, whether the feed XML file exists, how many valid prepared feed-entry
sidecars target that feed, and how many feed-entry sidecars are invalid. The
Publish pane renders that state in the preview status and dry-run detail. When
invalid feed-entry sidecars exist, the dry-run detail now includes the
sidecar path and validation issue text. It does not generate feeds
automatically.

The Publish pane also exposes a read-only validation-detail action for selected
invalid feed-entry diagnostics. It derives the recording ID from the diagnostic
sidecar path, calls `record validate-feed-entry --json`, and renders the
validator's errors and warnings. It does not edit or repair metadata.

The Publish pane also exposes a UI-local handoff from the selected invalid
feed-entry diagnostic to the Create pane. It selects the same project, derives
the recording ID from the diagnostic sidecar filename, fills the Create-pane
recording ID field, and leaves repair to the existing local Create controls.

The `publish --completed-history --json` command builds an inspectable
completed publication-history result record from the same dry-run plan as
planned history plus explicit transfer result, verification result, and
rollback fields. `publish --save-completed-history --json` writes that result
under the selected project `history/completed/` directory. `publish
--list-completed-history --json` and `publish --read-completed-history --json`
list and read saved records constrained to that same project-local directory.
`waystone-publishd` exposes non-mutating completed-history result-record
generation through `BuildCompletedHistory`; it does not write the record.
These commands and the D-Bus method are local result-recording contracts only;
they do not transfer files, verify remotes, or mutate remote systems. The Qt
Publish pane exposes completed-history list/read views through the local CLI
adapter, but does not create completed records.

The `publish --validate --json` command produces a non-mutating publication
readiness report with `valid`, `blocked`, `errors`, and `warnings`. The
validator checks project validation results, host and identity resolution,
enabled-feed readiness, invalid feed-entry sidecars, empty file-change plans,
and required confirmations. `waystone-publishd` exposes the same report through
`ValidatePublication`. This does not compare remote state, unlock credentials,
transfer files, or verify remote results.

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

Result after minimal Atom feed generation from prepared sidecars: relevant
checks passed on 2026-07-19, including Rust tests, clippy, CLI JSON contract
smoke, audiod smoke checks, and Qt project smoke.

Result after Publish-pane feed readiness reporting: relevant checks passed on
2026-07-19, including Rust tests, clippy, CLI JSON contract smoke, publishd
smoke checks, and Qt smoke checks.

Result after real `ffmpeg/libopus` Opus publication-copy export: relevant
checks passed on 2026-07-20, including Rust tests, clippy, CLI JSON contract
smoke, Qt build, focused Qt project smoke, and broad Qt smoke.

Result after Publish-pane invalid feed-entry diagnostics: relevant checks
passed on 2026-07-20, including Rust tests, clippy, CLI JSON contract smoke,
publishd D-Bus smoke, and focused Qt project smoke.

Result after Publish-pane feed-entry validation detail action: relevant checks
passed on 2026-07-20, including Qt build and focused Qt project smoke.

Result after Qt recording metadata update controls: relevant checks passed on
2026-07-20, including Qt build and focused Qt project smoke.

Result after feed-entry update command: relevant checks passed on 2026-07-20,
including Rust tests, clippy with warnings denied, CLI JSON contract smoke, and
audiod D-Bus smoke.

Result after Qt feed-entry update controls: relevant checks passed on
2026-07-20, including Qt build and focused Qt project smoke.

Result after Publish-to-Create feed diagnostic handoff: relevant checks passed
on 2026-07-20, including Qt build and focused Qt project smoke.

Result after local Atom feed merge/update: relevant checks passed on
2026-07-20, including Rust tests, clippy with warnings denied, CLI JSON
contract smoke, focused Qt project smoke, and broad Qt smoke.

Result after narrow local recording capture: relevant checks passed on
2026-07-20, including Rust tests, clippy with warnings denied, CLI JSON
contract smoke, audiod D-Bus smoke, focused Qt project smoke, broad Qt smoke,
format checks, and git diff whitespace checks.

Result after Qt recording capture controls: relevant checks passed on
2026-07-20, including Qt build, focused Qt project smoke, broad Qt smoke, Rust
tests, clippy with warnings denied, CLI JSON contract smoke, format checks, and
git diff whitespace checks.

Result after audiod local audio/feed D-Bus methods: relevant checks passed on
2026-07-20, including Rust tests, clippy with warnings denied, CLI JSON
contract smoke, audiod D-Bus smoke, focused Qt project smoke, broad Qt smoke,
format checks, and git diff whitespace checks.

Result after completed publication-history result records: relevant checks
passed on 2026-07-20, including Rust tests, clippy with warnings denied, CLI
JSON contract smoke, publishd D-Bus smoke, focused Qt project smoke, broad Qt
smoke, format checks, and git diff whitespace checks.

Result after publication readiness validation: relevant checks passed on
2026-07-20, including Rust tests, clippy with warnings denied, CLI JSON
contract smoke, publishd D-Bus smoke, focused Qt project smoke, broad Qt smoke,
format checks, and git diff whitespace checks.

Result after Qt publication validation display: relevant checks passed on
2026-07-20, including Rust tests, clippy with warnings denied, CLI JSON
contract smoke, publishd D-Bus smoke, focused Qt project smoke, broad Qt smoke,
format checks, and git diff whitespace checks.

Result after publishd completed-history generation: focused checks passed on
2026-07-20, including publish-service/publishd Rust tests, Rust workspace
tests, clippy with warnings denied, CLI JSON contract smoke, publishd D-Bus
smoke, focused Qt project smoke, broad Qt smoke, format checks, and git diff
whitespace checks.

Result after Qt completed-history read-only display: focused checks passed on
2026-07-20, including Qt build, focused Qt project smoke, Rust workspace tests,
clippy with warnings denied, CLI JSON contract smoke, publishd D-Bus smoke,
broad Qt smoke, format checks, and git diff whitespace checks.

## Important Boundaries

- Initial repository commit and push were completed after explicit user approval.
- No files outside this repository were edited by the assistant.
- Sibling Waystone applications remain future add-ons only.
- `waystone-projectd` direct D-Bus serving is implemented for project create, list, inspect, and validate.
- `waystone-publishd` direct D-Bus serving is implemented for non-mutating publication preview, publication readiness validation, planned-history generation, and completed-history result-record generation.
- `waystone-projectd` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-projectd` D-Bus service file and systemd user unit are present in the repo.
- `waystone-publishd` fails cleanly without a session bus and rejects duplicate bus ownership.
- `waystone-publishd` D-Bus service file and systemd user unit are present in the repo.
- `waystone-hostd` direct D-Bus serving is implemented for host list, inspect, and validate.
- `waystone-identityd` direct D-Bus serving is implemented for identity list, inspect, and validate.
- `waystone-audiod` direct D-Bus serving is implemented for recording metadata list, inspect, validate, attachment, update, capture, Opus export, feed-entry preparation/update, publication/feed-entry validation, and feed generation.
- `record attach` creates local audio metadata sidecars under a project's configured `[audio].metadata` root without copying audio, transcoding, generating feeds, or overwriting existing sidecars.
- `record update` rewrites existing local audio metadata sidecars in place while preserving the embedded recording ID, sidecar path, and optional measurement fields.
- `record capture` creates local WAV masters under a project's configured `[audio].masters` root from explicit `ffmpeg` input sources without overwriting existing files.
- `record export-opus` creates an encoded local Opus publication-copy file from an existing project-local master through `ffmpeg/libopus` without overwriting existing outputs.
- `record prepare-feed-entry` creates local feed-entry metadata sidecars under `feeds/entries/` without generating or updating feed XML.
- `record update-feed-entry` rewrites existing local feed-entry metadata sidecars under `feeds/entries/` without generating or updating feed XML.
- `record validate-publication` and `record validate-feed-entry` validate local audio publication handoff metadata without mutating files.
- `record generate-feed` creates minimal local Atom feed XML from validated `feeds/entries/*.toml` sidecars, replaces matching existing Atom entries by ID, and preserves unrelated existing Atom entries without publishing remotely.
- `publish --dry-run` reports feed readiness and invalid feed-entry diagnostics without generating or publishing feeds.
- `publish --validate` reports publication readiness without comparing remote state, transferring files, unlocking credentials, or verifying remotes.
- The Qt Publish pane displays `publish --validate` results for the selected project and target without mutating projects or remotes.
- `publish --completed-history` and `publish --save-completed-history` create local completed history result records from explicit result fields and do not execute remote transfer or verification.
- The Qt Publish pane lists and reads saved completed history records without creating completed records or mutating remotes.
- The Qt Publish pane can hand selected invalid feed-entry diagnostics back to the Create pane without editing metadata.
- The Qt Create pane exposes `record capture` through local CLI adapters for selected projects before Opus export or recording attachment.
- The Qt Create pane exposes `record attach` through local CLI adapters for selected projects with audio metadata configured.
- The Qt Create pane exposes `record update` through local CLI adapters for selected project recording sidecars.
- The Qt Create pane exposes `record export-opus` through local CLI adapters for selected projects before recording attachment.
- The Qt Create pane exposes feed-entry preparation, feed-entry update, publication/feed-entry validation status, and feed XML generation through local CLI adapters.
- `project create` scaffolds audio/feed defaults for `audio-series` and `mixed-publication` projects.
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

1. Decide whether completed-history save/list/read should be exposed through `waystone-publishd` D-Bus or remain CLI-local for now.
2. Decide whether the next local publish slice should add remote-state comparison scaffolding without executing transfer.
3. Decide whether Qt Publish-pane completed-history ergonomics need any small follow-up before moving back to service contracts.

Alternative next step:

- Continue Qt ergonomics only if it directly unblocks the local 0.1 demonstrable flow.

## Pause Marker

Current after Qt completed-history read-only display on 2026-07-20. The
latest handoff has been resumed and superseded by this checkpoint.
