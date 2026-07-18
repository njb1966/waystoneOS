# WaystoneOS Audio Metadata

Status: Draft for Phase 1 planning
Date: 2026-07-17

Audio metadata describes recordings and publication copies without requiring audio decoding or device access.

## Scope

Current implementation supports:

- TOML sidecar loading
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

## Sidecar Shape

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

## CLI Mapping

Current commands:

```text
record list ROOT
record inspect PATH
record validate PATH
listen library ROOT
```

The `record` command owns recording metadata inspection. The `listen` command can list playable recording metadata, but it does not play audio yet.
