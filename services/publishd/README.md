# waystone-publishd

`waystone-publishd` owns non-mutating publication preview and planned-history generation.

Current D-Bus service:

```text
org.waystone.Publish1
/org/waystone/Publish
```

Implemented methods:

```text
PreviewPublication
BuildPlannedHistory
```

The current daemon does not compare remote state, transfer files, delete files, verify remote results, unlock credentials, or write completed history records.

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
