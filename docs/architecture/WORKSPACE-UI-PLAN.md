# Waystone Workspace UI Plan

Status: Current scaffold plan
Date: 2026-07-18

This document defines the first Waystone Workspace UI planning boundary. It is not an implementation plan for a full desktop shell or custom compositor.

## Purpose

The first UI should prove that WaystoneOS can present an operating environment organized around projects, publications, hosts, identities, recordings, and workspaces without exposing a normal Linux desktop.

## Initial Frame

The version 0.1 workspace frame should include:

- Top menu bar
- Left activity navigation
- Main workspace area
- Bottom status bar
- Four workspace selectors: Explore, Create, Publish, Operate

Conceptual layout:

```text
System  Project  Publish  Network  Audio  Window  Help
-------------------------------------------------------
Explore        | Active workspace
Write          |
Listen         |
Record         |
Publish        |
Host           |
Connect        |
Learn          |
---------------|
Hosts          |
Services       |
Transfers      |
Terminal       |
-------------------------------------------------------
Create   Audio: Idle   Network: Offline   Project: None
```

## First Screens

### Explore

Purpose:

- Local small-web introduction
- Saved resources
- Protocol dispatcher placeholder
- Future Waystone Browser launch point

Do not implement a general web browser here.

### Create

Purpose:

- Project list
- Project inspection
- Project validation
- Gemtext file placeholder
- Audio attachment placeholder

Backed by:

- `crates/project-format`
- `crates/project-service`
- `project` CLI behavior

### Publish

Purpose:

- Select project
- Select target
- Show dry-run transfer plan
- Show host and identity resolution state
- Show required confirmations
- Show planned publication history record

Backed by:

- `crates/publish-plan`
- `crates/publication-history`
- `crates/host-identity`

No remote mutation in the first UI.

### Operate

Purpose:

- Host list
- Host inspection
- Identity list
- Identity inspection
- Future Waystone Comm launch point

Backed by:

- `crates/host-identity`
- `host` CLI behavior
- `identity` CLI behavior

## Visual Rules

- Compact rectangular controls
- Visible borders
- Clear focus state
- No hover-only essential actions
- No mobile-style cards
- No translucent panels
- No animated docks
- No search-only navigation
- Text labels for unfamiliar actions
- Fit 1366x768
- Preserve keyboard operation

## Implementation Boundary

The first UI prototype uses Qt 6 C++ on Debian, as accepted in ADR-0012.

It should not:

- Implement a custom compositor
- Modify Browser, Helm, or Comm repositories
- Require D-Bus to be complete
- Depend on network access
- Perform remote publishing
- Expose package management

The UI should consume the same libraries and command semantics already defined for CLI operations.

## Current Scaffold

The first source scaffold exists in:

```text
ui/workspace-qt/
```

It currently renders the operating frame and wires the Create pane to `project`,
`record`, and `listen` CLI JSON output for project and recording metadata. The
Create pane can create projects under the configured projects root, including
audio/feed scaffold defaults for audio-capable project types, add removable
publish-target metadata, refresh the project list, open the new project in the
editor, load, edit, save, validate, locally preview, and locally link-check the
selected project's content index file, list files under the selected project's
content root, filter that file list, show read-only detail for the selected
content file, attach existing project-local audio files through `record attach
--json`, create mock publication copies through `record export-opus --json`,
prepare feed-entry sidecars through `record prepare-feed-entry --json`, show
publication/feed-entry validation status through `record validate-publication
--json` and `record validate-feed-entry --json`, and generate feed XML through
`record generate-feed --json`. The Publish pane lists
configured local projects, derives target choices from project metadata, filters
visible projects by project name, ID, type, path, or target names, offers a
target selector for projects with multiple targets, shows
ready/blocked/failed/feed-aware preview status, uses read-only `publish
--dry-run --json` previews, renders feed readiness and invalid feed-entry
diagnostics from dry-run output, shows a
compact per-target overview for the selected project, lets overview row
selection update the active target and existing preview panes, shows planned
publication history file-action summaries plus raw records through `publish
--planned-history --json`, saves planned preview records under project
`history/previews/` through `publish --save-planned-history-preview --json`,
lists saved preview records through `publish --list-planned-history-previews
--json`, loads selected saved preview TOML through `publish
--read-planned-history-preview --json`, preserves the selected saved-preview row
across refreshes when the preview still exists, reports first-line differences
between generated and selected saved planned history, and filters visible
saved-preview records by filename or path. The Operate pane uses read-only
`host` and `identity` CLI JSON output for list, inspect, and validate.

The UI accepts local root configuration through `--config` using an INI file. Without an explicit config file, it checks Qt's user app config location for `workspace.ini`; if absent, it defaults to repository examples. The Explore pane displays the active roots and can write persistent user root settings for later launches. The same binary also exposes a local `--check-roots` diagnostic mode for startup and smoke-test verification.

Explore resource rows still use placeholder data.
