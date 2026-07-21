# Version 0.1 Development Milestone

Status: Draft for Phase 0
Date: 2026-07-17

Version 0.1 is the Debian-hosted development preview for WaystoneOS.

It exists to prove the operating model before bootable image work begins.

## Milestone Goal

Build a dedicated Wayland session on Debian that demonstrates WaystoneOS as an operating environment organized around projects, publishing, hosts, identities, recordings, and workspaces.

The milestone should validate the OS foundation without embedding sibling Waystone applications as the center of the work.

## Workstream 1: Workspace Frame

Deliver:

- Dedicated Wayland session entry
- Waystone Workspace frame
- Four workspaces: Explore, Create, Publish, Operate
- Activity navigation
- Status area for audio, network, project, and workspace state
- Compact workstation-style visual treatment
- Keyboard navigation and visible focus

Can be mocked:

- Network state
- Service state
- Add-on app launch targets
- Publication state

Must be real:

- Workspace navigation behavior
- Visual hierarchy
- Low-resolution layout discipline

## Workstream 2: Project Format and Service Contract

Deliver:

- `.wayproject` fixture
- `project.toml` schema-1 example
- Manifest parser design
- Validation rules
- Project service API sketch
- CLI contract for `project create`, `project list`, `project inspect`, and `project validate`

Can be mocked:

- D-Bus activation
- GUI data source

Must be real:

- Project format documentation
- Validation behavior definition
- Test fixtures once code begins
- Initial project-format parser and validator

## Workstream 3: CLI Foundation

Deliver:

- Native command naming policy
- Common flags
- Exit code table
- JSON response shape
- Error message rules
- Dry-run and confirmation rules

Can be mocked:

- Command backends

Must be real:

- CLI standards before implementation

## Workstream 4: Publishing Model

Deliver:

- Publishing target model
- Dry-run transfer model
- Validation checklist
- Remote verification model
- Publication history record shape
- Safety rules for remote deletion

Can be mocked:

- SSH-family remote transfer execution
- Remote verification

Must be real:

- Dry-run semantics
- Destructive action confirmation rules
- Credential separation model

## Workstream 5: Audio Path

Deliver:

- Audio workflow contract
- Recording metadata model
- Opus export expectation
- Project attachment behavior
- Feed enclosure handoff

Can be mocked:

- Device enumeration
- Recording engine
- Export engine

Must be real:

- Audio scope boundary
- Master-versus-publication-copy model
- Metadata and attachment model

## Workstream 6: Host and Identity Model

Deliver:

- Host record fields
- Identity record fields
- SSH host-key trust behavior
- Certificate inspection expectations
- Credential export warning behavior

Can be mocked:

- Actual secret storage
- Real SSH sessions

Must be real:

- Separation between project metadata and credentials
- Trust-state vocabulary

## Workstream 7: Add-On Integration Points

Deliver:

- Browser launch point definition
- Helm project-state consumption point
- Comm host/session handoff point
- Protocol dispatcher responsibility sketch

Can be mocked:

- App launches
- App APIs
- Cross-repository behavior

Must be real:

- OS-owned service contracts
- Clear statement that add-on apps do not define the OS architecture

## Acceptance Criteria

Version 0.1 planning is ready for implementation when:

- The project format is documented.
- Service ownership for the first slice is documented.
- CLI standards are documented.
- The workspace frame has clear responsibilities.
- Publishing dry-run semantics are defined.
- Audio attachment and export scope are defined.
- Host and identity boundaries are defined.
- Add-on app integration is documented without touching sibling repos.
- Security and accessibility baselines are represented in the milestone.

Version 0.1 implementation is complete when:

- A user can understand the WaystoneOS model without seeing a normal Linux desktop.
- A project can be represented by an inspectable `.wayproject`.
- The UI and CLI refer to the same project concepts.
- A dry-run publication can show what would happen without changing a remote host.
- Audio can be modeled as a master plus publication copy attached to a project.
- The prototype advances the first vertical slice.

Current implementation progress is tracked in [IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md).
Manual preview steps for the repo-local development session are tracked in
[MANUAL-PREVIEW-CHECKLIST.md](MANUAL-PREVIEW-CHECKLIST.md).

## Explicit Deferrals

Do not include in version 0.1:

- Bootable USB image
- Yocto image work
- Custom compositor
- Encrypted persistence implementation
- A/B updates
- Internal-drive installer
- Classroom mode
- Full hosting stack
- Full DAW behavior
- Cross-repository app modification
