# waystone-audiod

`waystone-audiod` owns recording metadata listing, inspection, validation, and
local audio/feed mutation over D-Bus. The daemon adapts the existing audio
service crate contract for local sidecar attachment, existing sidecar update,
WAV master capture from explicit `ffmpeg` input sources, Opus publication-copy
export through `ffmpeg/libopus`, feed-entry sidecar preparation and refresh,
publication-copy validation, feed-entry validation, and local Atom feed
generation. It does not enumerate audio devices, play audio, publish remotely,
or install activation files.

Current D-Bus service:

```text
org.waystone.Audio1
/org/waystone/Audio
```

Implemented methods:

```text
ListRecordings
InspectRecording
ValidateRecording
AttachRecording
UpdateRecording
CaptureRecording
ExportOpus
PrepareFeedEntry
UpdateFeedEntry
ValidatePublication
ValidateFeedEntry
GenerateFeed
```

Audio device enumeration, playback, editing, codec transcoding beyond Opus
publication export, and codec inspection are not implemented.

## Activation Files

Repository activation artifacts:

```text
services/audiod/dbus/org.waystone.Audio1.service
services/audiod/systemd/waystone-audiod.service
```

Install locations for a user-session install:

```text
$XDG_DATA_HOME/dbus-1/services/org.waystone.Audio1.service
$XDG_CONFIG_HOME/systemd/user/waystone-audiod.service
```

System-wide package installs should use the distribution-appropriate equivalents:

```text
/usr/share/dbus-1/services/org.waystone.Audio1.service
/usr/lib/systemd/user/waystone-audiod.service
```

The checked-in activation files assume the daemon binary is installed at:

```text
/usr/bin/waystone-audiod
```

## Verification

Direct daemon and method smoke:

```bash
scripts/audiod-dbus-smoke.sh
```

D-Bus autostart smoke using a generated temporary service file and the repo build artifact:

```bash
scripts/audiod-dbus-activation-smoke.sh
```

Systemd user unit syntax smoke using a temporary daemon path:

```bash
scripts/audiod-systemd-unit-smoke.sh
```

The smoke scripts use private or temporary paths and do not install files outside the repository.
