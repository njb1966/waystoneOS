# Phase 0 and Version 0.1 Alignment

Status: current after Phase 0/0.1 alignment audit
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

The main remaining 0.1 gap is the middle of the first vertical slice: audio
attachment, publication-copy expectation, and feed-enclosure handoff. The repo
can inspect audio metadata and include published audio/feed files in dry-run
plans, but it does not yet provide a concrete local workflow for attaching an
existing recording/publication copy to a project or preparing feed metadata.

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
| Project Format and Service Contract | Strong | Format, examples, validation, create/list/inspect/validate CLI, service wrapper, and D-Bus adapter exist. Project repair, migration, and archive/export are not implemented. |
| CLI Foundation | Strong | Core CLIs use stable command names, human output, JSON output, shared error envelope, and integration tests. `way` is command discovery only, not dispatch. |
| Publishing Model | Strong for dry-run scope | Dry-run plans, blocked states, planned history generation, saved preview records, and Publish-pane inspection exist. Remote comparison, transfer, verification, and completed history are not implemented. |
| Audio Path | Partial, next gap | Metadata sidecars, validation, record/listen CLIs, audio service boundary, and D-Bus adapter exist. Audio capture, playback, Opus export, attachment workflow, and feed generation are not implemented. |
| Host and Identity Model | Strong for metadata scope | Host/identity records, validation, CLIs, service wrappers, D-Bus adapters, and Operate-pane read-only inspection exist. Secret storage and SSH host probing are not implemented. |
| Add-On Integration Points | On track | Browser, Helm, and Comm remain add-on integration targets. No sibling repositories have been modified. |

## Minimum Demonstrable Flow

Version 0.1 scope defines this local flow:

| Flow Step | Current State |
| --- | --- |
| Open or create a project | Implemented through CLI and Qt Create pane |
| Write Gemtext | Implemented for selected content index |
| Preview locally | Implemented as local Create-pane preview and link validation |
| Record or attach an audio file | Partially modeled through metadata fixtures and record CLI inspection |
| Export an Opus publication copy | Not implemented; only existing published-copy paths are modeled |
| Generate or update feed metadata | Not implemented; existing feed files can be included in dry-run plans |
| Configure a host/destination | Partially implemented through examples, host/identity metadata, and removable targets |
| Run publication validation | Partially implemented through project, audio, host, identity, and dry-run validation |
| Perform a dry-run publish | Implemented for local plans without remote mutation |
| Show publication history or planned transfer state | Implemented as planned history previews and saved preview records |

This points to audio attachment and feed handoff as the most useful next 0.1
slice. It advances the vertical workflow without starting real recording,
codec export, packaging, installed services, remote transfer, or compositor work.

## Deliberate Next Slice

Recommended next implementation slice:

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
- Add CLI/service behavior that can create or update an audio metadata sidecar
  under the selected project without escaping the project root.
- Preserve the master-versus-publication-copy model.
- Represent feed enclosure handoff in metadata without generating a full feed
  engine yet.
- Add focused tests and, if the CLI contract is stable, expose the workflow in
  the Create pane later.

This is more aligned than another UI refinement because it fills a documented
vertical-slice gap. It is also safer than packaging, installed D-Bus activation,
real SSH transfer, or compositor work because those are explicitly deferred or
not yet necessary for validating the 0.1 operating model.

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
