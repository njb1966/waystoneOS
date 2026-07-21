# ADR-0015: Debian Session Install Layout

Status: Accepted
Date: 2026-07-21

## Context

Version 0.1 is a Debian-hosted development preview that should run as a
dedicated Wayland session before any bootable image or installer work begins.

The repository already contains the Qt Workspace binary source, Rust CLIs,
service binaries, D-Bus service files, and systemd user unit source artifacts.
Those artifacts are still repo-local. Installing them into `/usr`, user service
directories, or XDG session directories would cross from implementation
contract into host mutation.

The next OS-facing step needs to define the session and install layout without
changing the developer machine.

## Decision

WaystoneOS version 0.1 will use a repo-local Debian session/install-layout
contract before any installer or package automation.

The source layout is:

- `session/waystone.desktop`: source XDG Wayland session entry
- `session/waystone-session`: source launcher wrapper
- `session/install-layout.toml`: source manifest for future installed paths
- `scripts/session-layout-smoke.sh`: non-installing verification

The future installed layout for the Debian-hosted preview is:

- `/usr/share/wayland-sessions/waystone.desktop`
- `/usr/bin/waystone-session`
- `/usr/bin/waystone-workspace`
- `/usr/bin/waystone-projectd`
- `/usr/bin/waystone-publishd`
- `/usr/bin/waystone-hostd`
- `/usr/bin/waystone-identityd`
- `/usr/bin/waystone-audiod`
- `/usr/share/dbus-1/services/org.waystone.Project1.service`
- `/usr/share/dbus-1/services/org.waystone.Publish1.service`
- `/usr/share/dbus-1/services/org.waystone.Host1.service`
- `/usr/share/dbus-1/services/org.waystone.Identity1.service`
- `/usr/share/dbus-1/services/org.waystone.Audio1.service`
- `/usr/lib/systemd/user/waystone-projectd.service`
- `/usr/lib/systemd/user/waystone-publishd.service`
- `/usr/lib/systemd/user/waystone-hostd.service`
- `/usr/lib/systemd/user/waystone-identityd.service`
- `/usr/lib/systemd/user/waystone-audiod.service`

The launcher wrapper executes the Workspace binary selected by
`WAYSTONE_WORKSPACE_BIN`, falling back to `/usr/bin/waystone-workspace`. It
fails with exit code `127` and a clear diagnostic when the selected Workspace
binary is not executable.

The wrapper may pass explicit repo-development settings through
`WAYSTONE_REPO_ROOT` and `WAYSTONE_WORKSPACE_CONFIG`, but it does not create
directories, install files, start services, call D-Bus, or mutate system state.

## Consequences

- The dedicated-session contract is inspectable and smoke-testable in the repo.
- Later packaging or installer work has stable target paths to implement.
- Developers can validate wrapper behavior without installing files outside the
  repository or launching the real GUI.
- The `.desktop` session entry is not sufficient by itself until a later slice
  installs it into a display-manager-visible session directory.
- Installed D-Bus activation remains deferred even though repo-local service
  files and user unit source artifacts already exist.

## Alternatives Considered

- Install the session entry and wrapper immediately under `/usr`. Rejected for
  this slice because the project owner explicitly limited current work to
  repo-local artifacts.
- Use `$XDG_DATA_HOME/wayland-sessions` for a user-local development install.
  Deferred because it still mutates host session state and should be handled by
  an explicit installer/package or dev-install slice.
- Launch the Qt Workspace directly from the display manager. Rejected because a
  wrapper gives the OS session a stable executable name and one place to manage
  future environment setup.

## Follow-Up

- Add a dev-install or package slice only after explicit approval.
- Add installed-session smoke coverage once installation into an approved
  temporary root or host path exists.
- Keep compositor, boot image, credential unlock, remote verification, and
  sibling application embedding deferred.
