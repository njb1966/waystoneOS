# Architecture Decision Register

This directory records significant WaystoneOS architecture and product decisions.

Use ADRs for decisions that affect:

- Product identity
- Release format
- User-visible architecture
- Service boundaries
- IPC contracts
- Persistent data formats
- Security boundaries
- Boot, update, or recovery behavior
- Toolkit, compositor, image, or language choices
- Major workflow scope

Do not use ADRs for routine implementation details, small bug fixes, or temporary experiments that do not change project direction.

## Status Values

- Proposed: under discussion
- Accepted: current project direction
- Superseded: replaced by a later ADR
- Rejected: considered and intentionally not adopted

## Register

| ADR | Status | Title |
| --- | --- | --- |
| [0001](0001-product-identity-and-release-format.md) | Accepted | Product identity and first release format |
| [0002](0002-debian-prototype-before-yocto-image.md) | Accepted | Prototype on Debian before Yocto image construction |
| [0003](0003-shared-services-for-gui-and-cli.md) | Accepted | Shared services for graphical and CLI behavior |
| [0004](0004-waystone-workspace-visual-direction.md) | Accepted | Waystone Workspace visual direction |
| [0005](0005-first-party-application-independence.md) | Accepted | First-party application independence |
| [0006](0006-audio-scope-for-version-1.md) | Accepted | Audio scope for version 1 |
| [0007](0007-wlroots-compositor-sequencing.md) | Accepted | wlroots compositor sequencing |
| [0008](0008-immutable-system-and-encrypted-persistence.md) | Accepted | Immutable system and encrypted persistence |
| [0009](0009-licensing-policy.md) | Accepted | Licensing policy |
| [0010](0010-os-first-before-add-on-app-integration.md) | Accepted | OS-first before add-on app integration |
| [0011](0011-project-format-v0.md) | Accepted | Project format v0 |
| [0012](0012-workspace-ui-prototype-route.md) | Accepted | Workspace UI prototype route |

## Creating a New ADR

Copy [template.md](template.md), assign the next four-digit number, and use a short kebab-case filename.
