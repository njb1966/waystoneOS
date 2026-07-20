# WaystoneOS Audio Metadata

Status: Current local metadata contract
Date: 2026-07-20

Audio metadata describes recordings and publication copies without requiring audio decoding or device access.

## Scope

Current implementation supports:

- TOML sidecar loading
- TOML sidecar creation for attaching an existing master/publication copy
- Opus publication-copy export from an existing project-local master file through `ffmpeg/libopus`
- TOML feed-entry sidecar preparation from existing recording metadata
- Recording listing
- Recording inspection
- Metadata validation
- Publication-copy and feed-entry handoff validation
- Project-relative master and published paths
- Feed enclosure metadata references
- Minimal Atom feed XML generation from validated feed-entry sidecars
- Existing Atom feed update that replaces managed entries by entry ID and preserves unrelated existing entries

Current implementation does not support:

- PipeWire device enumeration
- Recording capture
- Playback
- Trimming
- Normalization
- Codec export or transcoding beyond Opus publication copies
- Audio codec inspection
- Waveform generation
- Multi-format feed generation
- Full metadata merge editing beyond the narrow update commands

## Recording Sidecar Shape

Audio sidecars live under:

```text
audio/metadata/
```

Example:

```toml
[recording]
id = "field-note"
title = "Field Note"
master = "audio/masters/field-note.flac"
published = "audio/published/field-note.opus"
duration_seconds = 318
channels = 1
sample_rate = 48000

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:field-note"
mime_type = "audio/ogg; codecs=opus"
```

Paths are project-relative and must not be absolute or traverse upward with `..`.

## Feed Entry Sidecar Shape

Prepared feed-entry sidecars live under:

```text
feeds/entries/
```

Example:

```toml
[entry]
id = "tag:example.invalid,2026:field-note"
title = "Field Note"
updated = "2026-07-19T00:00:00Z"
summary = "Field note summary"
feed = "feeds/feed.xml"
recording = "field-note"
recording_metadata = "audio/metadata/field-note.toml"

[enclosure]
path = "audio/published/field-note.opus"
mime_type = "audio/ogg; codecs=opus"
```

This is a preparation contract for publication tools. It records the feed entry
and enclosure fields consumed by the minimal Atom feed generator.

## CLI Mapping

Current commands:

```text
record attach PROJECT ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record update PROJECT RECORDING_ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record export-opus PROJECT MASTER PUBLISHED PRESET
record prepare-feed-entry PROJECT RECORDING_ID UPDATED SUMMARY
record update-feed-entry PROJECT RECORDING_ID UPDATED SUMMARY
record validate-publication PROJECT RECORDING_ID
record validate-feed-entry PROJECT RECORDING_ID
record generate-feed PROJECT
record list ROOT
record inspect PATH
record validate PATH
listen library ROOT
```

`record attach` creates one metadata sidecar under the selected project's
configured `[audio].metadata` root. It references an existing project-relative
master file, an existing project-relative publication copy, and a feed enclosure
handoff path. It does not copy files, record audio, generate a feed, or
overwrite an existing sidecar.

`record update` rewrites an existing recording sidecar selected from the
project's configured `[audio].metadata` root. It preserves the existing
`recording.id`, sidecar path, and optional measurement fields
`duration_seconds`, `channels`, and `sample_rate`, while replacing title,
master, publication-copy, feed, entry ID, and MIME fields. It requires the new
master and publication-copy paths to be existing project-relative files. It
does not edit audio, create a new sidecar, update prepared feed-entry sidecars,
or merge feed XML.

`record export-opus` creates an Opus publication copy from an existing
project-local master file through `ffmpeg` with the `libopus` encoder. It
validates project-relative paths, accepts a narrow preset name, requires the
output path to end with `.opus`, writes through a temporary output, and refuses
to overwrite an existing publication copy. The JSON response reports
`mime_type = "audio/ogg; codecs=opus"` and `engine = "ffmpeg"`. The command
does not record audio, edit the master file, update metadata sidecars, or
publish remotely.

`record prepare-feed-entry` creates one feed-entry sidecar under
`feeds/entries/` from an existing recording sidecar in the selected project's
configured `[audio].metadata` root. It requires the recording sidecar to include
`recording.published`, `publication.feed`, `publication.entry_id`, and
`publication.mime_type`, and it requires the published audio file to exist. It
does not generate or modify the feed XML file by itself.

`record update-feed-entry` rewrites an existing `feeds/entries/<recording-id>.toml`
sidecar from the current recording sidecar and a new `UPDATED`/`SUMMARY` pair.
It refreshes title, entry ID, feed path, enclosure path, and MIME type from the
recording metadata. It does not create missing feed-entry sidecars, generate
feed XML by itself, or publish remotely.

`record validate-publication` checks an existing recording sidecar in project
context. It validates the referenced master file, publication-copy file,
required publication fields, project-relative feed path, and MIME shape.

`record validate-feed-entry` checks a prepared feed-entry sidecar in project
context. It validates required entry and enclosure fields, verifies referenced
recording metadata and enclosure audio exist, checks that feed-entry values
match the recording sidecar's publication fields, and reports duplicate feed
entry IDs in `feeds/entries/`.

`record generate-feed` reads the selected project's `[feed]` manifest section,
supports enabled Atom feeds only, validates every `feeds/entries/*.toml`
sidecar, sorts sidecar-managed entries by descending `entry.updated`, and
atomically writes the configured feed path. When the configured feed already
contains Atom entries, matching IDs are replaced from sidecars and unrelated
existing entries are preserved in their current XML form. The feed title, feed
ID, and top-level updated value remain generated from project configuration and
entry timestamps. It is intentionally a minimal local generator: it does not
merge remote feeds, support RSS, or expose D-Bus.

The `record` command owns recording metadata creation and inspection. The
`listen` command can list playable recording metadata, but it does not play
audio yet.
