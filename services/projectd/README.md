# waystone-projectd

`waystone-projectd` owns project creation, listing, inspection, and validation.

Current D-Bus service:

```text
org.waystone.Project1
/org/waystone/Project
```

Implemented methods:

```text
CreateProject
ListProjects
InspectProject
ValidateProject
```

## Activation Files

Repository activation artifacts:

```text
services/projectd/dbus/org.waystone.Project1.service
services/projectd/systemd/waystone-projectd.service
```

Install locations for a user-session install:

```text
$XDG_DATA_HOME/dbus-1/services/org.waystone.Project1.service
$XDG_CONFIG_HOME/systemd/user/waystone-projectd.service
```

System-wide package installs should use the distribution-appropriate equivalents:

```text
/usr/share/dbus-1/services/org.waystone.Project1.service
/usr/lib/systemd/user/waystone-projectd.service
```

The checked-in activation files assume the daemon binary is installed at:

```text
/usr/bin/waystone-projectd
```

## Verification

Direct daemon and method smoke:

```bash
scripts/projectd-dbus-smoke.sh
```

D-Bus autostart smoke using a generated temporary service file and the repo build artifact:

```bash
scripts/projectd-dbus-activation-smoke.sh
```

Systemd user unit syntax smoke using a temporary daemon path:

```bash
scripts/projectd-systemd-unit-smoke.sh
```

The smoke scripts use private or temporary paths and do not install files outside the repository.
