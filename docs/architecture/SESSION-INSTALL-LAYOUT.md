# Session Install Layout

Status: Current as of 2026-07-21

This document defines the repo-local Debian session and future install layout
for the version 0.1 development preview.

Version 0.1 is a dedicated Wayland session on Debian. This repository now
contains the source artifacts that describe that session, but it still does not
install anything into system or user session directories.

## Source Artifacts

```text
session/
  install-layout.toml
  waystone.desktop
  waystone-session
scripts/session-layout-smoke.sh
```

`session/waystone.desktop` is the source XDG session entry. In a later
installer or package slice, it is expected to become:

```text
/usr/share/wayland-sessions/waystone.desktop
```

`session/waystone-session` is the source launcher wrapper. In a later installer
or package slice, it is expected to become:

```text
/usr/bin/waystone-session
```

`session/install-layout.toml` is the source-of-truth manifest for the expected
Debian preview paths. It is documentation and smoke-test input only; it is not
an installer.

## Launcher Contract

The session entry launches:

```text
/usr/bin/waystone-session
```

The wrapper resolves the Workspace binary in this order:

1. `WAYSTONE_WORKSPACE_BIN`
2. `/usr/bin/waystone-workspace`

If the selected binary is not executable, the wrapper exits `127` and prints a
clear diagnostic.

For repo-local development checks, the wrapper accepts these optional
environment variables:

- `WAYSTONE_REPO_ROOT`: passed to the Workspace as `--repo-root VALUE`
- `WAYSTONE_WORKSPACE_CONFIG`: passed to the Workspace as `--config VALUE`

The wrapper does not create directories, install files, start services, call
D-Bus, unlock credentials, launch sibling applications, or mutate remote
systems.

## Future Installed Paths

The version 0.1 Debian-hosted preview expects these installed paths when a
later install/package slice is approved:

```text
/usr/share/wayland-sessions/waystone.desktop
/usr/bin/waystone-session
/usr/bin/waystone-workspace
/usr/bin/waystone-projectd
/usr/bin/waystone-publishd
/usr/bin/waystone-hostd
/usr/bin/waystone-identityd
/usr/bin/waystone-audiod
/usr/share/dbus-1/services/org.waystone.Project1.service
/usr/share/dbus-1/services/org.waystone.Publish1.service
/usr/share/dbus-1/services/org.waystone.Host1.service
/usr/share/dbus-1/services/org.waystone.Identity1.service
/usr/share/dbus-1/services/org.waystone.Audio1.service
/usr/lib/systemd/user/waystone-projectd.service
/usr/lib/systemd/user/waystone-publishd.service
/usr/lib/systemd/user/waystone-hostd.service
/usr/lib/systemd/user/waystone-identityd.service
/usr/lib/systemd/user/waystone-audiod.service
```

These paths are a contract. They are not installed by the current repository
state.

## Verification

Run:

```bash
scripts/session-layout-smoke.sh
```

The smoke script verifies:

- the session desktop entry names the WaystoneOS session
- the session entry points to `/usr/bin/waystone-session`
- the install manifest lists the expected future target paths
- the wrapper is executable
- the wrapper passes repo-development settings to a fake Workspace binary
- the wrapper fails clearly when the selected Workspace binary is missing

The smoke script writes only to a temporary directory under `/tmp`.

## Deferrals

The current session layout does not provide:

- installation into `/usr` or `$XDG_DATA_HOME`
- Debian package metadata
- display-manager registration
- installed D-Bus activation
- bootable image generation
- a custom compositor
- credential unlock
- SSH transfer execution
- remote verification
- audio device enumeration or playback
- sibling Waystone application embedding
