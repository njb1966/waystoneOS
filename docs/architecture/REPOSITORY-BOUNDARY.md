# WaystoneOS Repository Boundary

Status: Draft for Phase 0
Date: 2026-07-17

This repository is for WaystoneOS itself.

## Current Work Boundary

For the current OS-first phase, work stays inside:

```text
/media/nick/1TB_Storage1/projects/waystone-repos/waystoneOS
```

Sibling Waystone application directories may exist outside this repository, but they are not part of the current working surface unless the project owner explicitly asks for cross-repository work.

## Add-On App Position

Waystone Browser, Waystone Helm, and Waystone Comm are first-party Waystone applications.

In the current phase, WaystoneOS treats them as:

- Future integration targets
- Add-on applications
- Consumers of WaystoneOS services
- Standalone-capable tools that should not define the OS architecture

WaystoneOS should first define:

- OS concepts
- Workspace model
- Service boundaries
- CLI contracts
- Project format
- Publishing model
- Identity and host model
- Security boundaries

Only then should integration with add-on applications become concrete.

## What Not To Do Yet

Do not:

- Modify sibling app repositories
- Import app source code into this repository
- Make WaystoneOS depend on the current implementation details of sibling apps
- Treat launching Browser, Helm, or Comm as sufficient OS integration
- Build the OS as a wrapper around existing applications

## Acceptable References

It is acceptable to reference Browser, Helm, and Comm in architecture documents when defining:

- Integration points
- Expected service APIs
- Protocol dispatch behavior
- Host/session handoff
- Project state consumption
- Future native WaystoneOS profiles

Those references should not require editing the sibling repositories.

