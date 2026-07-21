# waystone-publishd

`waystone-publishd` owns publication preview, publication readiness validation,
transfer-intent reporting, confirmed local/removable execution,
planned-history generation, and local completed-history result-record
generation, save, list, and read operations.

Current D-Bus service:

```text
org.waystone.Publish1
/org/waystone/Publish
```

Implemented methods:

```text
PreviewPublication
ValidatePublication
TransferIntent
ExecuteRemovable
BuildPlannedHistory
BuildCompletedHistory
SaveCompletedHistory
ListCompletedHistory
ReadCompletedHistory
```

The current daemon can use caller-supplied local remote-state manifests for
preview, transfer-intent comparison, and confirmed removable execution.
`ExecuteRemovable` is limited to local/removable file-copy execution, requires
`confirm_transfer = true`, rejects unsupported request fields, writes completed
history from executor results, and keeps verification as `not-run`. It does not
probe remote state, execute SSH-family transfers, delete files, verify remote
results, or unlock credentials. `SaveCompletedHistory` writes only a
caller-requested project-local record under `history/completed/`.

## Activation Files

Repository activation artifacts:

```text
services/publishd/dbus/org.waystone.Publish1.service
services/publishd/systemd/waystone-publishd.service
```

Install locations for a user-session install:

```text
$XDG_DATA_HOME/dbus-1/services/org.waystone.Publish1.service
$XDG_CONFIG_HOME/systemd/user/waystone-publishd.service
```

System-wide package installs should use the distribution-appropriate equivalents:

```text
/usr/share/dbus-1/services/org.waystone.Publish1.service
/usr/lib/systemd/user/waystone-publishd.service
```

The checked-in activation files assume the daemon binary is installed at:

```text
/usr/bin/waystone-publishd
```

## Verification

Direct daemon and method smoke:

```bash
scripts/publishd-dbus-smoke.sh
```

D-Bus autostart smoke using a generated temporary service file and the repo build artifact:

```bash
scripts/publishd-dbus-activation-smoke.sh
```

Systemd user unit syntax smoke using a temporary daemon path:

```bash
scripts/publishd-systemd-unit-smoke.sh
```

The smoke scripts use private or temporary paths and do not install files outside the repository.
