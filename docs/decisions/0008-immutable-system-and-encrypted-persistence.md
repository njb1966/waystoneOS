# ADR-0008: Immutable System and Encrypted Persistence

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS is a portable live USB system. Users may carry it between machines and may lose the physical device. Updates must be recoverable, and user data must remain portable without allowing arbitrary mutation of the base system.

## Decision

The normal WaystoneOS base system should be read-only or image-based.

User data belongs in an encrypted persistent workspace. Temporary sessions must be supported with no retained keys or history.

The intended system layout is:

```text
EFI system partition
System image A
System image B
Recovery image
Encrypted workspace
Optional shared export partition
```

The update model should use signed A/B images with health checks and automatic rollback.

## Consequences

- Visible package management is excluded from normal mode.
- Arbitrary base-system mutation belongs in Workshop Mode, not normal operation.
- Workspace migration and recovery are core product features.
- Failed updates must not leave the user without a bootable system.
- Credentials must be excluded from ordinary project exports.

## Alternatives Considered

- Mutable package-managed live system: rejected because it conflicts with reproducibility, portability, and recovery.
- Unencrypted persistence: rejected because stolen USB devices are an expected threat.
- Single system image with no rollback: rejected because update failure would be too costly.

## Follow-Up

- Evaluate RAUC, OSTree, systemd-sysupdate, or equivalent update mechanisms.
- Define workspace unlock, recovery key, export, and clean shutdown behavior.
- Define which settings are system state versus workspace state.

