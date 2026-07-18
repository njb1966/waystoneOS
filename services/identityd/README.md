# waystone-identityd

`waystone-identityd` owns identity listing, inspection, and validation.

Current D-Bus service:

```text
org.waystone.Identity1
/org/waystone/Identity
```

Implemented methods:

```text
ListIdentities
InspectIdentity
ValidateIdentity
```

## Activation Files

Repository activation artifacts:

```text
services/identityd/dbus/org.waystone.Identity1.service
services/identityd/systemd/waystone-identityd.service
```

Install locations for a user-session install:

```text
$XDG_DATA_HOME/dbus-1/services/org.waystone.Identity1.service
$XDG_CONFIG_HOME/systemd/user/waystone-identityd.service
```

System-wide package installs should use the distribution-appropriate equivalents:

```text
/usr/share/dbus-1/services/org.waystone.Identity1.service
/usr/lib/systemd/user/waystone-identityd.service
```

The checked-in activation files assume the daemon binary is installed at:

```text
/usr/bin/waystone-identityd
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
