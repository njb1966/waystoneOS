# waystone-audiod

`waystone-audiod` owns recording metadata listing, inspection, and validation.
The audio service crate also supports local sidecar attachment for `record
attach`, but that mutating operation is not exposed over D-Bus yet.

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
```

Audio capture, playback, editing, codec inspection, and device enumeration are not implemented.

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
