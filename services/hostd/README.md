# waystone-hostd

`waystone-hostd` owns host listing, inspection, and validation.

Current D-Bus service:

```text
org.waystone.Host1
/org/waystone/Host
```

Implemented methods:

```text
ListHosts
InspectHost
ValidateHost
```

## Activation Files

Repository activation artifacts:

```text
services/hostd/dbus/org.waystone.Host1.service
services/hostd/systemd/waystone-hostd.service
```

Install locations for a user-session install:

```text
$XDG_DATA_HOME/dbus-1/services/org.waystone.Host1.service
$XDG_CONFIG_HOME/systemd/user/waystone-hostd.service
```

System-wide package installs should use the distribution-appropriate equivalents:

```text
/usr/share/dbus-1/services/org.waystone.Host1.service
/usr/lib/systemd/user/waystone-hostd.service
```

The checked-in activation files assume the daemon binary is installed at:

```text
/usr/bin/waystone-hostd
```

## Verification

Direct daemon and method smoke:

```bash
scripts/host-identity-dbus-smoke.sh
```

D-Bus autostart smoke using generated temporary service files and repo build artifacts:

```bash
scripts/host-identity-dbus-activation-smoke.sh
```

Systemd user unit syntax smoke using temporary daemon paths:

```bash
scripts/host-identity-systemd-unit-smoke.sh
```

The smoke scripts use private or temporary paths and do not install files outside the repository.
