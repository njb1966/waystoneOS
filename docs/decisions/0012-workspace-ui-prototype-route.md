# ADR-0012: Workspace UI Prototype Route

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS needs a first Workspace UI that can exercise projects, publishing dry-runs, host metadata, identity metadata, recordings, and workspace navigation without becoming a general Linux desktop or a custom compositor project.

The UI should fit Debian prototype development, remain local-first, preserve keyboard operation, and avoid embedding the sibling Browser, Helm, and Comm repositories during the OS-first phase.

## Decision

The first Workspace UI prototype will use Qt 6 with C++ on Debian.

The Qt application should be a thin UI shell over WaystoneOS service contracts. It may call current CLI JSON output or use a narrow adapter to service crates during early prototyping, but domain behavior must remain in Rust crates and command/service contracts.

The first UI prototype will not use Rust Qt bindings as the primary route, and it will not treat an HTML mockup as the production UI path.

## Consequences

- Qt 6 gives the project a mature widget toolkit, accessibility baseline, keyboard behavior, and Wayland support without requiring a custom compositor.
- C++ keeps the Qt layer close to the toolkit's native documentation and packaging model.
- The UI layer must stay thin. Widgets do not own project parsing, publishing decisions, host trust rules, identity validation, or audio metadata validation.
- Tests and manual checks must include 1366x768 layout, keyboard navigation, and low-resource/offline behavior.
- Rust Qt bindings may be revisited later if they become a better maintenance tradeoff.
- HTML sketches may still be used for disposable design exploration, but not as the committed Workspace implementation route.

## Alternatives Considered

- Rust Qt bindings: deferred because binding maturity and packaging risk are not worth taking before the service contracts are proven.
- HTML UI shell: rejected as the production route because Waystone Workspace is an OS environment, not a web app, and should avoid introducing a browser runtime dependency for its core shell.
- Custom compositor first: rejected for version 0.1 because compositor work is sequenced after proving the Debian prototype and service-driven Workspace.

## Follow-Up

- Add a minimal `ui/workspace-qt/` scaffold only after the UI source layout is approved.
- Define how the first Qt screen reads service/CLI data before implementing widgets.
- Keep Browser, Helm, and Comm as future add-on launch/integration targets, not embedded dependencies.
