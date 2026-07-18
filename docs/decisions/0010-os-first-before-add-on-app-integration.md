# ADR-0010: OS-First Before Add-On App Integration

Status: Accepted
Date: 2026-07-17

## Context

Waystone Browser, Helm, and Comm are part of the Waystone product family and will eventually be included with WaystoneOS. They currently exist as separate application efforts outside this repository.

The immediate project goal is the actual operating system foundation, not embedding existing tools or modifying sibling application repositories.

## Decision

WaystoneOS development will proceed OS-first.

During the current phase, work remains inside the WaystoneOS repository unless the project owner explicitly requests cross-repository changes.

Browser, Helm, and Comm are treated as add-on applications and future integration targets. WaystoneOS should define the services, contracts, project model, identity model, host model, publication model, and workspace behavior those applications can consume later.

## Consequences

- The OS architecture will not be shaped around current implementation details of sibling apps.
- Integration points can be documented without requiring immediate app changes.
- The first implementation work should focus on WaystoneOS-owned components.
- Launching first-party apps is not sufficient to prove OS integration.
- Future cross-repository work must be explicit.

## Alternatives Considered

- Start by embedding existing apps: rejected because it risks producing an app bundle instead of an operating environment.
- Modify sibling app repositories immediately: rejected because the OS service contracts are not defined yet.
- Ignore the apps entirely: rejected because they remain part of the product family and important integration targets.

## Follow-Up

- Define service APIs before requiring app integration.
- Keep workflow maps clear about which steps are OS-owned and which will later involve add-on apps.
- Revisit app integration after the project, publishing, host, identity, and audio contracts are stable enough to consume.

