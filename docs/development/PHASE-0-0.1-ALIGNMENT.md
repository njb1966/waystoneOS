# Phase 0 and Version 0.1 Alignment

Status: current after repo-local session/install-layout contract
Date: 2026-07-21

This document records the deliberate alignment check between the Phase 0
architecture baseline, the version 0.1 development-preview scope, and the
current repository implementation.

Reviewed sources:

- `PLAN.md`
- `docs/architecture/VERSION-0.1-SCOPE.md`
- `docs/development/VERSION-0.1-MILESTONE.md`
- `docs/development/IMPLEMENTATION-STATUS.md`
- `docs/development/CHECKPOINT.md`
- `docs/development/PUBLISH-TRANSFER-READINESS-AUDIT.md`
- Current repository file layout

## Summary Judgment

WaystoneOS remains on track against the Phase 0 and version 0.1 plan.

Phase 0 is effectively serving as the current architecture baseline: charter,
scope, decisions, service boundaries, CLI standards, project format, publishing
model, host/identity model, audio metadata model, and repository boundary are
all documented in-repo.

Version 0.1 implementation has advanced beyond pure planning in several
foundational areas:

- Inspectable `.wayproject` format and validation
- Native CLI contracts with JSON output
- Non-mutating publish dry-runs, planned history previews, and local completed
  history result records through CLI/service/D-Bus construction and persistence
  paths
- Non-mutating publication readiness validation through CLI, service crate, and
  publishd D-Bus
- Qt Publish-pane display for publication readiness validation and read-only
  completed-history record list/detail
- Host, identity, and audio metadata inspection boundaries
- D-Bus adapter binaries and repo-local activation artifacts
- Qt Workspace prototype with Explore, Create, Publish, and Operate panes

The main alignment decision after the Qt removable-state export helper is that
the local app/service foundation is now strong enough for version 0.1 planning
purposes. Additional publishing-pane refinements would be useful but are no
longer the best next step. The next required project movement should pivot
toward the OS/session layer: define the Debian-hosted session entry, install
layout, binary/service placement, and verification path needed to run the
current Workspace as WaystoneOS rather than as an ordinary manually launched Qt
application.

The most recent slices filled the first local part of the audio/feed gap: the
repo can now create audio-capable project scaffolds for `audio-series` and
`mixed-publication`, create an audio metadata sidecar that attaches an existing
master and publication copy to a project, record feed enclosure handoff fields,
prepare a feed-entry sidecar under `feeds/entries/`, and expose attachment in
the Qt Create pane. It also validates publication-copy and feed-entry handoff
metadata in project context and exposes feed-entry preparation plus validation
status in the Qt Create pane. It also has a minimal local Atom feed generator
from validated `feeds/entries/` sidecars. It also has a real
`ffmpeg/libopus` publication-copy export command for existing project-local
master files and exposes that command in the Qt Create pane. It can also update
an existing recording sidecar's descriptive and publication fields while
preserving the sidecar path, embedded recording ID, and optional measurement
fields, and it exposes that update command in the Qt Create pane. It can also refresh an
existing prepared feed-entry sidecar from current recording metadata and expose
that feed-entry update command in the Qt Create pane. The Publish pane can also
hand a selected invalid feed-entry diagnostic back to the Create pane with the
matching project and derived recording ID loaded. It can now capture a WAV
master from an explicit local `ffmpeg` input source, and the Qt Create pane
exposes that capture command. It still does not enumerate audio devices,
play audio, publish remotely, or install service activation files.

## Phase 0 Alignment

Phase 0 deliverables from the master plan are covered:

| Phase 0 Item | Current State |
| --- | --- |
| Project charter | `docs/architecture/CHARTER.md` |
| Product boundary | `docs/architecture/REPOSITORY-BOUNDARY.md` and ADR 0010 |
| Architecture-decision register | `docs/decisions/` |
| Version 0.1 scope | `docs/architecture/VERSION-0.1-SCOPE.md` |
| Development milestone | `docs/development/VERSION-0.1-MILESTONE.md` |
| Service boundaries | `docs/architecture/SERVICE-BOUNDARIES.md` and service docs |
| CLI standards | `docs/architecture/CLI-STANDARDS.md` |
| Security baseline | `docs/architecture/SECURITY-MODEL.md` |

No Phase 0 document currently requires a broad rewrite before implementation
continues. Future changes should be surgical and tied to implemented behavior
or approved scope changes.

## Version 0.1 Workstream Alignment

| Workstream | Alignment | Notes |
| --- | --- | --- |
| Workspace Frame | Partial, now next critical path | Four panes exist in Qt. Navigation, visual frame, root config, focused smoke coverage, repo-local Debian session/install-layout artifacts, wrapper-to-Workspace dev-session smoke coverage, and temporary-root install-layout validation are real. Actual installation, display-manager registration, installed activation, and terminal integration remain deferred. |
| Project Format and Service Contract | Strong | Format, examples, validation, type-specific audio/feed creation defaults, create/list/inspect/validate CLI, service wrapper, and D-Bus adapter exist. Project repair, migration, and archive/export are not implemented. |
| CLI Foundation | Strong | Core CLIs use stable command names, human output, JSON output, shared error envelope, and integration tests. `way` is command discovery only, not dispatch. |
| Publishing Model | Strong for dry-run, validation, local comparison, transfer-intent, local-history, removable preparation, and local removable execution scope | Dry-run plans, local remote-state export/inspection helpers, removable destination-state export, caller-supplied local remote-state comparison, non-mutating transfer-intent reports through CLI/service/D-Bus, removable executor preparation through CLI/service, confirmed local/removable file-copy execution through CLI/service/D-Bus, failed/partial copy-time executor history, D-Bus removable executor smoke coverage, Qt transfer-intent display, Qt removable execution readiness display, Qt comparison display/input, Qt removable destination-state export helper, publication readiness validation, Qt validation display, feed readiness reporting with invalid feed-entry diagnostics, selected diagnostic validation detail, diagnostic handoff back to Create, blocked states, planned history generation, saved preview records, completed history result records, D-Bus completed-history result-record generation/save/list/read, Qt completed-history list/detail display, and Publish-pane inspection exist. Remote probing, SSH-family transfer, delete execution, and verification are not implemented. |
| Audio Path | Partial, improved | Audio-capable project defaults, recording metadata sidecars, recording metadata update, feed-entry metadata update, narrow WAV master capture from explicit `ffmpeg` input sources, real `ffmpeg/libopus` Opus publication-copy export, feed-entry metadata sidecars, publication/feed-entry handoff validation, minimal Atom feed XML generation with local existing-entry merge/update, local sidecar attachment, Qt Create-pane capture, export, attachment, recording-update, feed-entry preparation/update, validation, and feed-generation controls, record/listen CLIs, audio service boundary, and D-Bus adapter for local audio/feed operations exist. Audio device enumeration and playback are not implemented. |
| Host and Identity Model | Strong for metadata scope | Host/identity records, validation, CLIs, service wrappers, D-Bus adapters, and Operate-pane read-only inspection exist. Secret storage and SSH host probing are not implemented. |
| Add-On Integration Points | On track | Browser, Helm, and Comm remain add-on integration targets. No sibling repositories have been modified. |

## Minimum Demonstrable Flow

Version 0.1 scope defines this local flow:

| Flow Step | Current State |
| --- | --- |
| Open or create a project | Implemented through CLI and Qt Create pane, including audio/feed scaffolding for audio-capable project types |
| Write Gemtext | Implemented for selected content index |
| Preview locally | Implemented as local Create-pane preview and link validation |
| Record or attach an audio file | Narrow WAV master capture is implemented through `record capture` from explicit `ffmpeg` input sources and exposed in the Qt Create pane; attach is implemented for existing local master/publication-copy files through metadata sidecar creation and exposed in the Qt Create pane |
| Export an Opus publication copy | Implemented through `record export-opus` using `ffmpeg/libopus` from an existing project-local master and exposed in the Qt Create pane |
| Generate or update feed metadata | Implemented for local Atom feeds through feed-entry sidecar preparation/update, validation, minimal feed XML generation with existing-entry merge/update, and publish dry-run feed readiness reporting with invalid feed-entry diagnostics; non-Atom feed formats and remote feed merge are not implemented |
| Configure a host/destination | Partially implemented through examples, host/identity metadata, and removable targets |
| Run publication validation | Implemented as non-mutating publish readiness validation through CLI, service crate, and publishd D-Bus; caller-supplied local remote-state comparison is available, while remote probing and remote verification remain deferred |
| Perform a dry-run publish | Implemented for local plans, local remote-state manifest export/inspection, and caller-supplied local remote-state comparison without remote mutation |
| Show publication history or planned transfer state | Implemented as planned history previews, non-mutating transfer-intent reports, saved preview records, CLI-local completed history save/list/read, D-Bus completed-history result-record generation/save/list/read, and Qt read-only completed-history list/detail display |

The current 0.1 slice has connected prepared feed entries, minimal feed XML
generation with local existing-entry merge/update, Qt generation controls,
publish dry-run feed readiness reporting with invalid feed-entry diagnostics
and selected diagnostic validation detail, a real `ffmpeg/libopus` Opus
publication-copy command, Qt controls for that export, and Qt controls for
feed-entry sidecar update. It also has a small Publish-to-Create diagnostic
handoff for invalid feed-entry sidecars, a narrow local WAV capture contract
from explicit `ffmpeg` input sources, Qt Create-pane controls for that capture
command, D-Bus exposure for the local audio/feed service operations, and
CLI-local completed publication-history result records and D-Bus
completed-history result-record generation/save/list/read. The Qt Publish pane
can list and read saved completed records without creating them. It also has an
explicit non-mutating publication readiness validation report through the CLI,
service crate, publishd D-Bus, and Qt Publish pane. Dry-run preview can compare
local publishable files against a caller-supplied local remote-state manifest
and classify upload/delete/skip without contacting or mutating a remote. The Qt
Publish pane can provide that local manifest path and render comparison
metadata plus upload/update/delete/skip dry-run buckets. The Qt Publish pane
can also export a selected removable target's destination-state manifest into a
project-local helper file under `history/previews/` and load that path into the
existing comparison field. The CLI can also export the local publishable path
set as a plain text remote-state manifest and inspect existing local manifests
with the same parser. It can also export a removable target destination root's
current file path set into the same local manifest shape. The CLI and publish
service can now build a non-mutating
transfer-intent report with
`execution_ready`, blocking issues, required confirmations, host/identity
summaries, comparison metadata, change buckets, and the future completed-history
directory. `waystone-publishd` exposes the same read-only report through D-Bus,
and the Qt Publish pane displays the local CLI transfer-intent report. It
also has a non-mutating removable executor preparation contract through the CLI
and service crate that reports a bounded local destination root plus per-file
source/destination operation records while blocking unsupported methods,
existing transfer-intent blockers, and delete operations. It can now execute
confirmed local/removable file-copy transfers through the CLI and service crate,
refuse upload overwrites, and write completed history from executor results,
including failed/partial copy-time outcomes, while leaving verification as
`not-run`. `waystone-publishd` exposes the confirmed local/removable executor
through the ADR-0014 `ExecuteRemovable` method with private-session-bus smoke
coverage. It
also displays removable execution readiness in the Qt Publish pane through the
local CLI adapter without exposing `publish --execute-removable` in the UI. It
should still avoid device enumeration, packaging, installed services, remote
transfer, and compositor work.

## Deliberate Next Slice

Audit result:

The previous implementation slices left the publishing refinement lane and
started the Debian-hosted OS/session lane with a repo-local
session/install-layout contract plus wrapper-to-Workspace dev-session smoke
coverage and temporary-root install-layout validation before installing
anything outside the repository.

Recommended next deliverables:

- Add a narrow dev-run command for launching the current preview from the repo.
- Keep actual installation into `/usr/share`, `/usr/bin`,
  `$XDG_DATA_HOME`, or system/user service directories behind explicit
  approval and a later installer/package slice.
- Keep Qt mutating publish controls, SSH transfer, remote deletion, credential
  unlock, remote verification, audio device enumeration, and playback deferred.

Completed implementation slices:

**Audio publication attachment and feed handoff, local-only.**

The completed slices keep the current boundaries:

- No audio device enumeration
- No playback implementation
- No codec transcoding beyond the narrow Opus publication-copy export
- No remote publication mutation
- No installed service activation
- No sibling app changes

Concrete deliverables should be small and inspectable:

- Define the local operation for attaching an existing master/publication copy
  to a project using audio metadata sidecars.
- Add CLI/service behavior that can create an audio metadata sidecar under the
  selected project without escaping the project root.
- Preserve the master-versus-publication-copy model.
- Represent feed enclosure handoff in metadata without generating a full feed
  engine yet.
- Add focused tests.
- Expose the stable local attach workflow in the Qt Create pane.
- Add type-specific audio/feed scaffold defaults for `audio-series` and
  `mixed-publication` project creation.
- Add create-only feed-entry sidecar preparation from existing recording
  metadata, while keeping full feed XML generation deferred.
- Add local validation for publication-copy and feed-entry handoff metadata.
- Expose feed-entry preparation and validation status in the Qt Create pane
  through CLI adapters.
- Add minimal Atom feed XML generation from validated feed-entry sidecars.
- Preserve unrelated existing Atom entries while replacing sidecar-managed feed
  entries by ID.
- Expose minimal feed generation in the Qt Create pane through the CLI adapter.
- Report feed readiness in publish dry-runs and the Qt Publish pane.
- Report invalid feed-entry diagnostics in dry-run JSON, D-Bus preview JSON,
  and the Qt Publish pane.
- Add read-only validation detail for a selected invalid feed-entry diagnostic
  in the Qt Publish pane.
- Add real `ffmpeg/libopus` Opus publication-copy export from an existing
  project-local master.
- Expose Opus publication-copy export in the Qt Create pane.
- Add a small metadata replacement/update command for existing recording
  sidecars, while preserving recording ID, sidecar path, and optional
  measurement fields.
- Expose recording metadata update in the Qt Create pane.
- Add a narrow feed-entry update command for existing prepared sidecars.
- Expose feed-entry update in the Qt Create pane.
- Add a Publish-to-Create handoff for selected invalid feed-entry diagnostics.
- Add narrow WAV master capture from an explicit local `ffmpeg` input source.
- Expose narrow WAV master capture in the Qt Create pane.
- Expose existing local audio/feed service operations through `waystone-audiod`
  D-Bus.
- Add completed publication-history result records from explicit result fields
  and save/list/read them under project `history/completed/`.
- Expose completed-history result-record generation through `waystone-publishd`.
- Expose completed-history save/list/read through `waystone-publishd` for
  project-local completed history records.
- Show saved completed-history records in the Qt Publish pane through read-only
  CLI list/read adapters.
- Add a non-mutating publication readiness report for project, host/identity,
  feed, file-change, and confirmation readiness before remote execution.
- Add caller-supplied local remote-state comparison for dry-run previews,
  validation planning, history preview generation, and publishd preview JSON.
- Expose local remote-state comparison input and change-bucket display in the
  Qt Publish pane.
- Add local remote-state manifest export/inspection helpers in the publish CLI.
- Add removable destination-state manifest export in the publish CLI.
- Audit transfer-readiness gates before any real remote mutation.
- Add a non-mutating transfer-intent report in the publish planning/service/CLI
  path before any real transfer implementation.
- Expose the transfer-intent report through `waystone-publishd` as a read-only
  D-Bus method.
- Display the transfer-intent report in the Qt Publish pane through the local
  CLI adapter.
- Define the bounded removable executor preparation contract and CLI/service
  test harness before file-copy behavior.
- Implement confirmed local/removable file-copy execution with a
  temporary-project test harness and completed history written from executor
  results.
- Display removable execution readiness in the Qt Publish pane through the
  local CLI adapter without adding a mutating execution control.
- Record failed and partial copy-time removable execution outcomes as
  executor-produced completed-history records.
- Define the future D-Bus removable executor request/response shape before
  implementing any mutating publish IPC method.
- Implement confirmed D-Bus removable execution in `waystone-publishd` from the
  ADR-0014 shape with private-session-bus smoke coverage.
- Add Qt read-only ergonomics for removable destination-state export into a
  project-local comparison manifest.

## Explicitly Still Deferred

The following remain intentionally out of scope for the next slice:

- Bootable USB image
- Yocto image construction
- Internal-drive installer
- Custom compositor
- Encrypted persistence implementation
- Installing files outside the repository
- Installed D-Bus activation
- Remote probing for comparison
- Real SSH transfer execution
- Remote verification
- Secret storage
- Audio device enumeration
- Audio playback engine
- Browser, Helm, or Comm repository modification

## Current Risk Posture

The main risk is not that the project is off track. The main risk is continuing
to deepen already-functional app/service lanes while the 0.1 milestone still
requires a dedicated Debian Wayland session entry before it can feel like an
operating environment.

The project should continue to prefer:

- CLI and service contracts before UI conveniences
- Local-only, inspectable files before remote mutation
- Narrow explicit audio capture/export contracts before device integration
- Repository-local artifacts before install automation
