# WaystoneOS Version 0.1 Scope

Status: Draft for Phase 0
Date: 2026-07-17

Version 0.1 is a development preview. It validates the operating model before bootable image work begins.

## Release Type

Version 0.1 runs as a dedicated Wayland session on Debian 13.

It is not:

- A bootable WaystoneOS image
- A production Yocto image
- A public stable release
- A custom compositor milestone
- An installer milestone

## Purpose

Version 0.1 should prove:

- The Waystone Workspace model is understandable.
- The four-workspace organization works.
- Waystone terminology is usable.
- GUI and CLI operations can share service contracts.
- Projects can be created, inspected, edited, validated, and prepared for publication.
- Basic audio recording can attach to a project.
- Publishing over SSH can be modeled safely.
- Add-on app integration points are clear without making those apps the center of the OS.

## Included

- Dedicated Wayland session on Debian
- Waystone Workspace frame
- Four workspaces: Explore, Create, Publish, Operate
- Activity navigation
- Project view
- Basic Gemtext editing path
- Local preview model
- Host and destination model
- Service mockups
- Audio status and basic recording path
- CLI framework
- Initial project service contract
- Initial publishing service contract
- Initial host and identity model
- Structured log convention
- Basic user documentation stubs

## Add-On App Policy for 0.1

Waystone Browser, Helm, and Comm are integration targets.

For 0.1, WaystoneOS should define:

- Where Browser would be launched from
- Where Helm would read project state from
- Where Comm would receive host/session data from
- What service APIs those apps would consume

WaystoneOS should not depend on modifying sibling application repositories during this phase unless the project owner explicitly asks.

## Excluded

- Bootable USB image
- Yocto layer
- Custom wlroots compositor
- Internal-drive installer
- A/B image updates
- Encrypted persistence implementation
- Full hosting stack
- Classroom mode
- General HTTP workstation behavior
- Visible package management
- Full DAW functionality
- Arbitrary plugin hosting
- ARM64 support
- Broad NVIDIA support

## Minimum Demonstrable Flow

The 0.1 preview should demonstrate this local flow, even if some pieces use mock services:

1. Open or create a project.
2. Write Gemtext.
3. Preview locally.
4. Record or attach an audio file.
5. Export an Opus publication copy.
6. Generate or update feed metadata.
7. Configure a host/destination.
8. Run publication validation.
9. Perform a dry-run publish.
10. Show publication history or planned transfer state.

Actual remote publication is desirable for 0.1 if it can be implemented safely without distorting the scope.

## Acceptance Criteria

Version 0.1 is acceptable when:

- Users can understand the WaystoneOS operating model without seeing a normal Linux desktop.
- The UI avoids generic desktop and mobile-card patterns.
- The CLI and GUI model the same project data.
- Project data is stored in inspectable files.
- Destructive publication behavior is dry-run first.
- Service boundaries are documented even where mocked.
- The next implementation phase can build real services without changing the product vocabulary.

## Risks

- Building too much UI before service contracts exist
- Turning the prototype into a generic Qt desktop app
- Treating add-on app launching as OS integration
- Letting publishing skip validation and dry-run behavior
- Letting audio expand into DAW scope
- Beginning compositor work before workflow validation

