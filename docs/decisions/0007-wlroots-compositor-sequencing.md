# ADR-0007: wlroots Compositor Sequencing

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS needs a custom graphical environment. Building a compositor too early could consume the project before the product workflows are validated.

wlroots provides mature Wayland building blocks. Smithay may be a future alternative, but maintaining two compositor implementations early would add unnecessary complexity.

## Decision

Use an existing wlroots compositor as scaffolding for early Debian-hosted prototypes.

Develop a dedicated wlroots compositor only after core workflows are validated.

Smithay remains a possible later alternative, but the project will not maintain simultaneous wlroots and Smithay implementations during early development.

The dedicated compositor should initially focus on practical workstation needs: XDG windows, layer shell, outputs, input, workspaces, locking, clipboard, drag and drop, hot-plugging, multi-monitor support, XWayland compatibility, and crash recovery.

## Consequences

- Phase 1 can focus on Waystone Workspace behavior rather than compositor internals.
- The prototype compositor must not define the public product.
- Compositor-specific abstraction should be added only where a real future backend boundary exists.
- Fancy animations, blur, compositor plugins, complex tiling, scriptable window rules, desktop widgets, and wallpaper suites are out of initial scope.

## Alternatives Considered

- Build the custom compositor first: rejected because it delays user workflow validation.
- Use a conventional desktop shell: rejected because it would shape the product around a generic Linux desktop model.
- Build simultaneous wlroots and Smithay backends: rejected as premature.

## Follow-Up

- Select the prototype wlroots compositor for Phase 1.
- Define the minimum dedicated compositor requirements before Phase 7 starts.

