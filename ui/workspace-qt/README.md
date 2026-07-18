# Waystone Workspace Qt Prototype

Status: local authoring prototype

This directory contains the first Qt 6 C++ scaffold for Waystone Workspace.

It is intentionally separate from the Rust workspace. The UI renders the operating frame and uses existing CLI JSON output through a small Qt adapter layer.

## Boundaries

- No D-Bus dependency yet
- No remote publication
- No Browser, Helm, or Comm embedding
- No domain behavior in widgets
- No generated assets or network access

Current UI data is limited to:

- `project create --json`
- `project list --json`
- `project inspect --json` for project metadata and selected content index location
- `project validate --json`
- `publish --dry-run --json`
- `host list --json`
- `host inspect --json`
- `host validate --json`
- `identity list --json`
- `identity inspect --json`
- `identity validate --json`
- `record list --json`
- `record inspect --json`
- `record validate --json`
- `listen library --json`
- local creation of minimal projects under the configured projects root
- local reads and writes of the selected project's content index file
- local Gemtext preview rendering in the Create pane

Future UI data should continue to come from WaystoneOS service contracts, current CLI JSON output, or a narrow adapter approved before implementation.

## Workspace Roots

By default, the UI reads from repository examples:

- `examples/projects`
- `examples/connections/hosts`
- `examples/connections/identities`
- `examples/projects/audio-capsule.wayproject/audio/metadata`

An INI file can override those roots. Explicit `--config` wins. If no explicit config is passed, the UI checks Qt's user app config location for `workspace.ini`; if that file is absent, repository defaults are used.

The Explore pane can write persistent user root settings to the same user config file. Saved settings are picked up on the next launch unless `--config` or `--no-user-config` is used.

Example config:

```text
ui/workspace-qt/workspace.example.ini
```

## Debian Build

Expected packages:

```text
cmake
g++
qt6-base-dev
```

The Create, Publish, and Operate panes expect the current CLIs to be available. During development, build them first:

```bash
cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-host-cli -p waystone-identity-cli -p waystone-record-cli -p waystone-listen-cli
```

The UI looks for the matching binaries under `target/debug/` before falling back to commands on `PATH`.

Build commands:

```bash
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
```

Run command from the repository root:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace
```

Or pass the repository root explicitly:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
```

Run with explicit workspace roots:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --config /path/to/workspace.ini
```

Ignore user config and use repository defaults:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --no-user-config
```

Check configured roots without opening the window:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --check-roots
```

Repeatable smoke check:

```bash
scripts/workspace-qt-smoke.sh
```

CLI JSON contract smoke check:

```bash
scripts/cli-json-contract-smoke.sh
```
