# Phase 0 and Version 0.1 Alignment

Status: current after Qt feed-entry preparation controls
Date: 2026-07-19

This document records the deliberate alignment check between the Phase 0
architecture baseline, the version 0.1 development-preview scope, and the
current repository implementation.

Reviewed sources:

- `PLAN.md`
- `docs/architecture/VERSION-0.1-SCOPE.md`
- `docs/development/VERSION-0.1-MILESTONE.md`
- `docs/development/IMPLEMENTATION-STATUS.md`
- `docs/development/CHECKPOINT.md`
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
- Non-mutating publish dry-runs and planned history previews
- Host, identity, and audio metadata inspection boundaries
- D-Bus adapter binaries and repo-local activation artifacts
- Qt Workspace prototype with Explore, Create, Publish, and Operate panes

The most recent slices filled the first local part of the audio/feed gap: the
repo can now create audio-capable project scaffolds for `audio-series` and
`mixed-publication`, create an audio metadata sidecar that attaches an existing
master and publication copy to a project, record feed enclosure handoff fields,
prepare a feed-entry sidecar under `feeds/entries/`, and expose attachment in
the Qt Create pane. It also validates publication-copy and feed-entry handoff
metadata in project context and exposes feed-entry preparation plus validation
status in the Qt Create pane. It also has a minimal local Atom feed generator
from validated `feeds/entries/` sidecars. It still does not record audio,
export Opus, update or merge existing feed XML, expose feed generation over
D-Bus, or merge-update existing audio metadata.

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
| Workspace Frame | Partial, healthy | Four panes exist in Qt. Navigation, visual frame, root config, and focused smoke coverage are real. Dedicated Wayland session and terminal integration remain deferred. |
| Project Format and Service Contract | Strong | Format, examples, validation, type-specific audio/feed creation defaults, create/list/inspect/validate CLI, service wrapper, and D-Bus adapter exist. Project repair, migration, and archive/export are not implemented. |
| CLI Foundation | Strong | Core CLIs use stable command names, human output, JSON output, shared error envelope, and integration tests. `way` is command discovery only, not dispatch. |
| Publishing Model | Strong for dry-run scope | Dry-run plans, blocked states, planned history generation, saved preview records, and Publish-pane inspection exist. Remote comparison, transfer, verification, and completed history are not implemented. |
| Audio Path | Partial, improved | Audio-capable project defaults, recording metadata sidecars, feed-entry metadata sidecars, publication/feed-entry handoff validation, minimal Atom feed XML generation, local sidecar attachment, Qt Create-pane attachment/feed-entry/generate-feed controls, record/listen CLIs, audio service boundary, and D-Bus adapter exist. Audio capture, playback, Opus export, metadata replacement, existing feed merge/update, and D-Bus feed generation are not implemented. |
| Host and Identity Model | Strong for metadata scope | Host/identity records, validation, CLIs, service wrappers, D-Bus adapters, and Operate-pane read-only inspection exist. Secret storage and SSH host probing are not implemented. |
| Add-On Integration Points | On track | Browser, Helm, and Comm remain add-on integration targets. No sibling repositories have been modified. |

## Minimum Demonstrable Flow

Version 0.1 scope defines this local flow:

| Flow Step | Current State |
| --- | --- |
| Open or create a project | Implemented through CLI and Qt Create pane, including audio/feed scaffolding for audio-capable project types |
| Write Gemtext | Implemented for selected content index |
| Preview locally | Implemented as local Create-pane preview and link validation |
| Record or attach an audio file | Attach is implemented for existing local master/publication-copy files through metadata sidecar creation and exposed in the Qt Create pane; recording is not implemented |
| Export an Opus publication copy | Not implemented; only existing published-copy paths are modeled |
| Generate or update feed metadata | Partially implemented as create-only feed-entry sidecar preparation, validation, and minimal Atom feed XML generation; existing feed merge/update is not implemented |
| Configure a host/destination | Partially implemented through examples, host/identity metadata, and removable targets |
| Run publication validation | Partially implemented through project, audio, host, identity, and dry-run validation |
| Perform a dry-run publish | Implemented for local plans without remote mutation |
| Show publication history or planned transfer state | Implemented as planned history previews and saved preview records |

The current 0.1 slice starts a minimal feed XML generator from validated
`feeds/entries/` sidecars. The next slice should deliberately choose between
deeper publication integration or moving to the remaining audio-record/export
gap. It should still avoid real recording unless explicitly chosen, packaging,
installed services, remote transfer, and compositor work.

## Deliberate Next Slice

Completed implementation slices:

**Audio publication attachment and feed handoff, local-only.**

The slice should keep the current boundaries:

- No audio device capture
- No playback implementation
- No codec transcoding requirement
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
- Expose minimal feed generation in the Qt Create pane through the CLI adapter.

## Explicitly Still Deferred

The following remain intentionally out of scope for the next slice:

- Bootable USB image
- Yocto image construction
- Internal-drive installer
- Custom compositor
- Encrypted persistence implementation
- Installed D-Bus activation
- Real SSH transfer execution
- Remote verification
- Secret storage
- Audio device enumeration
- Audio recording engine
- Audio playback engine
- Opus transcoding engine
- Browser, Helm, or Comm repository modification

## Current Risk Posture

The main risk is not that the project is off track. The main risk is spending
too many slices on pane polish while the 0.1 vertical workflow still has a
clear audio/feed gap.

The project should continue to prefer:

- CLI and service contracts before UI conveniences
- Local-only, inspectable files before remote mutation
- Mocked or metadata-only audio behavior before real capture/export engines
- Repository-local artifacts before install automation
