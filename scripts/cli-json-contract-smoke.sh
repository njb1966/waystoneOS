#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo build \
  -p waystone-project-cli \
  -p waystone-publish-cli \
  -p waystone-host-cli \
  -p waystone-identity-cli \
  -p waystone-record-cli \
  -p waystone-listen-cli

python3 - "$repo_root" <<'PY'
import json
import subprocess
import sys

repo_root = sys.argv[1]

def run(command):
    completed = subprocess.run(
        command,
        cwd=repo_root,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return json.loads(completed.stdout)

def require(condition, message):
    if not condition:
        raise SystemExit(message)

project_list = run(["target/debug/project", "list", "--json", "examples/projects"])
projects = project_list["data"]["projects"]
require(projects and {"id", "name", "type", "path"} <= projects[0].keys(),
        "project list contract changed")

project_path = "examples/projects/audio-capsule.wayproject"
project_inspect = run(["target/debug/project", "inspect", "--json", project_path])
require({"id", "name", "type", "project_schema", "content_root", "content_index"}
        <= project_inspect["data"].keys(), "project inspect contract changed")
require("publish_targets" in project_inspect["data"],
        "project inspect publish target contract changed")

project_validate = run(["target/debug/project", "validate", "--json", project_path])
require("valid" in project_validate["data"], "project validate contract changed")

publish = run([
    "target/debug/publish", "--dry-run", "--project", project_path,
    "--target", "export", "--json"
])
require({"project", "target", "method", "destination", "blocked", "changes",
         "verification", "confirmations"} <= publish["data"].keys(),
        "publish dry-run contract changed")

planned_history = run([
    "target/debug/publish", "--planned-history", "--project", project_path,
    "--target", "export", "--date", "2026-07-18T00:00:00Z", "--json"
])
require({"project", "target", "transfer_result", "verification_result", "files", "record_toml"}
        <= planned_history["data"].keys(), "publish planned-history contract changed")
require(planned_history["data"]["transfer_result"] == "planned",
        "publish planned-history transfer result changed")
require(planned_history["data"]["files"] and
        {"path", "action"} <= planned_history["data"]["files"][0].keys(),
        "publish planned-history file contract changed")

host_list = run(["target/debug/host", "list", "--json", "examples/connections/hosts"])
hosts = host_list["data"]["hosts"]
require(hosts and {"id", "display_name", "address", "path"} <= hosts[0].keys(),
        "host list contract changed")

host_path = "examples/connections/hosts/offgridholdout.toml"
host_inspect = run(["target/debug/host", "inspect", "--json", host_path])
require({"id", "display_name", "address", "services"} <= host_inspect["data"].keys(),
        "host inspect contract changed")

identity_list = run(["target/debug/identity", "list", "--json", "examples/connections/identities"])
identities = identity_list["data"]["identities"]
require(identities and {"id", "display_name", "path"} <= identities[0].keys(),
        "identity list contract changed")

identity_path = "examples/connections/identities/nick-pub.toml"
identity_inspect = run(["target/debug/identity", "inspect", "--json", identity_path])
require({"id", "display_name", "ssh_keys", "certificates"} <= identity_inspect["data"].keys(),
        "identity inspect contract changed")

audio_root = "examples/projects/audio-capsule.wayproject/audio/metadata"
record_list = run(["target/debug/record", "list", "--json", audio_root])
recordings = record_list["data"]["recordings"]
require(recordings and {"id", "title", "path"} <= recordings[0].keys(),
        "record list contract changed")

record_path = "examples/projects/audio-capsule.wayproject/audio/metadata/field-note.toml"
record_inspect = run(["target/debug/record", "inspect", "--json", record_path])
require({"id", "title", "master", "published"} <= record_inspect["data"].keys(),
        "record inspect contract changed")

listen = run(["target/debug/listen", "library", "--json", audio_root])
playable = listen["data"]["recordings"]
require(playable and {"id", "title", "path"} <= playable[0].keys(),
        "listen library contract changed")

print("cli-json-contract smoke: all checked fields present")
PY
