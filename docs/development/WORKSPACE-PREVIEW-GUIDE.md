# Workspace Preview Guide

Status: Current as of 2026-07-21

This guide is for the current repo-local Waystone Workspace preview. It is not
an installed OS session, bootable image, or complete desktop environment yet.

## Golden Path

Use this path first. It is the current project-owner workflow.

1. Start the preview with a temporary config:

   ```bash
   scripts/run-dev-session.sh --config "$preview_config"
   ```

2. Open `Explore`.

   Confirm the active roots point at your temporary preview directory.

3. Open `Create`.

   Create a `capsule` project. Use a simple ID such as `temp1` and any short
   name.

4. In `Create > Write`, edit the Gemtext index.

   Save it and confirm the preview updates. For ordinary text-capsule testing,
   this is the main authoring surface.

5. In `Create > Files`, confirm the content files are listed.

   This is read-only inspection for the selected project's content root.

6. Open `Publish`.

   Select the project and target. Use `Refresh Preview` if the reports have not
   updated.

7. In `Publish > Preview`, confirm the dry-run and readiness reports are
   readable.

   This is still a preview. It does not publish remotely.

Stop there for the basic pass.

## Navigation Map

`Explore` is configuration and local root visibility. Use it first to confirm
that the preview is pointed at temporary roots.

`Create` is local project work. The `Write` tab is for Gemtext authoring. The
`Files` tab is read-only content inspection. The `Recordings` tab is optional
audio metadata work.

`Publish` is read-only publication preview and local history inspection. It
does not execute remote publishing from the UI.

`Operate` is local host and identity metadata inspection. Empty temporary roots
are expected during a basic preview.

The left navigation currently maps several future activities into these four
implemented panes. For example, `Write`, `Listen`, and `Record` open `Create`;
`Hosts`, `Services`, `Transfers`, and `Terminal` open `Operate`.

## What To Ignore For Now

Ignore `Create > Recordings` unless you are specifically checking the audio
metadata path. The current audio path is local and explicit-input only. It does
not enumerate audio devices or play audio.

Ignore `Publish > Feed` unless you are checking an audio/feed project. A text
capsule normally has no feed diagnostics.

Ignore `Publish > Planned History` unless you are checking local preview record
saving.

Ignore `Publish > Completed History` unless completed-history records have been
seeded by tests or local publish tooling.

Ignore `Operate` for the first golden-path pass unless you specifically want to
inspect host and identity metadata.

## Terms

`Preview` means a local dry-run report. It should tell you what would happen,
but it does not transfer files.

`Ready` means the local checks for that preview target passed.

`Blocked` means the preview found missing metadata, unsupported transfer
settings, invalid feed entries, or another local readiness problem.

`Compare State` is an optional local manifest used to compare expected
published paths. It is not live remote probing.

`Save Plan` writes a local planned-history preview record under the selected
project. It is not a completed publication record.

`Completed History` is a local record store for completed publish results. The
current UI reads those records but does not create them.

## Successful Basic Pass

A successful basic pass looks like this:

- `Explore` shows temporary roots as available.
- `Create` creates a `capsule` project.
- `Create > Write` lets you edit and save Gemtext.
- `Create > Files` shows project-local content files.
- `Publish` lists the project and an `export` target.
- `Publish > Preview` shows readable dry-run and readiness reports.
- No files are installed outside the repository or temporary preview roots.

