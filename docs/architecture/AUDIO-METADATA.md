# WaystoneOS Audio Metadata

Status: Current local metadata contract
Date: 2026-07-19

Audio metadata describes recordings and publication copies without requiring audio decoding or device access.

## Scope

Current implementation supports:

- TOML sidecar loading
- TOML sidecar creation for attaching an existing master/publication copy
- TOML feed-entry sidecar preparation from existing recording metadata
- Recording listing
- Recording inspection
- Metadata validation
- Project-relative master and published paths
- Feed enclosure metadata references

Current implementation does not support:

- PipeWire device enumeration
- Recording capture
- Playback
- Trimming
- Normalization
- Opus export
- Audio codec inspection
- Waveform generation
- Full feed XML generation or updates
- Metadata replacement or merge editing

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
and enclosure fields that a later feed generator needs, but it does not edit or
generate the feed XML file.

## CLI Mapping

Current commands:

```text
record attach PROJECT ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record prepare-feed-entry PROJECT RECORDING_ID UPDATED SUMMARY
record list ROOT
record inspect PATH
record validate PATH
listen library ROOT
```

`record attach` creates one metadata sidecar under the selected project's
configured `[audio].metadata` root. It references an existing project-relative
master file, an existing project-relative publication copy, and a feed enclosure
handoff path. It does not copy files, record audio, export Opus, generate a feed,
or overwrite an existing sidecar.

`record prepare-feed-entry` creates one feed-entry sidecar under
`feeds/entries/` from an existing recording sidecar in the selected project's
configured `[audio].metadata` root. It requires the recording sidecar to include
`recording.published`, `publication.feed`, `publication.entry_id`, and
`publication.mime_type`, and it requires the published audio file to exist. It
does not generate or modify the feed XML file.

The `record` command owns recording metadata creation and inspection. The
`listen` command can list playable recording metadata, but it does not play
audio yet.
