# ADR-0004: Waystone Workspace Visual Direction

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS needs a distinctive user interface that supports focused workstation use without becoming a conventional modern desktop or a retro costume.

The plan identifies Solaris, CDE, and OpenWindows as functional influences, not visual assets to copy.

## Decision

The graphical environment is called the Waystone Workspace.

It should use modern functional workstation organization informed by Solaris, CDE, and OpenWindows while avoiding literal imitation.

The interface should favor:

- Compact controls
- Visible borders
- Clear menus
- Strong keyboard access
- Predictable navigation
- High information density
- Minimal animation
- Workspaces
- Hosts and services as visible objects
- Restrained typography
- Rectangular and inset controls
- Clear focus indicators
- Text labels for unfamiliar actions

It should avoid:

- Mobile-style cards
- Oversized padding
- Floating translucent panels
- Blur
- Animated docks
- Search-only navigation
- Hover-only controls
- Decorative shadows
- Excessive rounded corners
- Consumer-cloud aesthetics
- Pixel-for-pixel historical imitation

## Consequences

- UI work must be tested at 1366x768 and HiDPI.
- Keyboard navigation and focus state are not optional.
- Essential actions must remain visible.
- WaystoneOS should not inherit GNOME, KDE, or mobile visual identity.
- The interface may look older than current consumer desktops if that improves clarity and density.

## Alternatives Considered

- Modern card-heavy desktop aesthetic: rejected because it conflicts with compact workstation use.
- Literal CDE clone: rejected because the goal is functional influence, not nostalgia.
- Search-first launcher model: rejected because navigable structure is a product requirement.

## Follow-Up

- Create UI standards for Waystone Workspace controls, menus, icons, focus, status, and dialogs.
- Validate early prototype screens with low-resolution and keyboard-only checks.

