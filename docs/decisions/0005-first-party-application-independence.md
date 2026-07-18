# ADR-0005: First-Party Application Independence

Status: Accepted
Date: 2026-07-17

## Context

Waystone Browser, Waystone Helm, and Waystone Comm are part of the Waystone product family. They need deeper integration inside WaystoneOS, but they should not become unusable outside the OS image.

Tight coupling would make the applications harder to test, distribute, and maintain.

## Decision

Waystone Browser, Helm, and Comm must remain independently usable on ordinary Linux systems.

Inside WaystoneOS, they may integrate with shared services for projects, identities, certificates, hosts, publishing, audio, libraries, permissions, and notifications.

Application core logic must not depend directly on WaystoneOS image internals.

## Consequences

- Integrations should be mediated through stable APIs.
- Standalone mode and WaystoneOS-native mode must be explicit.
- The OS profile of Waystone Browser may restrict HTTP use even if the standalone browser supports general HTTP and HTTPS.
- WaystoneOS cannot be defined as a bundle of these applications.

## Alternatives Considered

- Make applications WaystoneOS-only: rejected because it harms reuse and increases coupling.
- Keep applications fully separate with no OS integration: rejected because it prevents WaystoneOS from becoming a coherent operating environment.

## Follow-Up

- Define standalone versus native integration behavior for Browser, Helm, and Comm.
- Define protocol dispatch and service discovery between applications and Waystone services.

