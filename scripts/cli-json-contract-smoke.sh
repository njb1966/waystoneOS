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
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

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

with tempfile.TemporaryDirectory(prefix="waystone-project-create-contract-") as temp_root:
    audio_create = run([
        "target/debug/project", "create", "--json", temp_root,
        "contract-audio", "Contract Audio", "audio-series"
    ])
    require({"project_path", "project_schema"} <= audio_create["data"].keys(),
            "project create contract changed")
    audio_project = Path(audio_create["data"]["project_path"])
    require((audio_project / "audio" / "masters").is_dir(),
            "audio project create did not scaffold masters directory")
    require((audio_project / "audio" / "published").is_dir(),
            "audio project create did not scaffold published directory")
    require((audio_project / "audio" / "metadata").is_dir(),
            "audio project create did not scaffold metadata directory")
    require((audio_project / "feeds" / "feed.xml").is_file(),
            "audio project create did not scaffold feed placeholder")

publish = run([
    "target/debug/publish", "--dry-run", "--project", project_path,
    "--target", "export", "--json"
])
require({"project", "target", "method", "destination", "blocked", "changes",
         "verification", "confirmations", "feed"} <= publish["data"].keys(),
        "publish dry-run contract changed")
require({"configured", "enabled", "type", "path", "exists", "prepared_entries",
         "invalid_entries", "invalid_entry_diagnostics"} <= publish["data"]["feed"].keys(),
        "publish dry-run feed contract changed")

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

with tempfile.TemporaryDirectory(prefix="waystone-cli-json-contract-") as temp_root:
    temp_project = Path(temp_root) / "audio-capsule.wayproject"
    shutil.copytree(Path(repo_root) / project_path, temp_project)
    saved_history = run([
        "target/debug/publish", "--save-planned-history-preview", "--project", str(temp_project),
        "--target", "export", "--date", "2026-07-19T00:00:00Z", "--json"
    ])
    require({"project", "target", "output_path", "transfer_result", "verification_result", "files"}
            <= saved_history["data"].keys(),
            "publish save planned-history preview contract changed")
    output_path = Path(saved_history["data"]["output_path"])
    require(output_path.exists(), "publish save planned-history preview did not write output")
    require((temp_project / "history" / "previews") in output_path.parents,
            "publish save planned-history preview escaped project previews directory")
    listed_history = run([
        "target/debug/publish", "--list-planned-history-previews",
        "--project", str(temp_project), "--json"
    ])
    require({"project_path", "previews"} <= listed_history["data"].keys(),
            "publish list planned-history previews contract changed")
    require(listed_history["data"]["previews"],
            "publish list planned-history previews did not report saved output")
    require({"path", "filename", "modified_unix", "size_bytes"}
            <= listed_history["data"]["previews"][0].keys(),
            "publish list planned-history preview item contract changed")
    read_history = run([
        "target/debug/publish", "--read-planned-history-preview",
        "--project", str(temp_project),
        "--preview", listed_history["data"]["previews"][0]["path"],
        "--json"
    ])
    require({"project_path", "path", "filename", "modified_unix", "size_bytes", "record_toml"}
            <= read_history["data"].keys(),
            "publish read planned-history preview contract changed")
    require("[publication]" in read_history["data"]["record_toml"],
            "publish read planned-history preview did not return record TOML")

    (temp_project / "audio" / "masters" / "second-note.flac").write_bytes(b"master")
    exported_opus = run([
        "target/debug/record", "export-opus", "--json", str(temp_project),
        "audio/masters/second-note.flac",
        "audio/published/second-note.opus",
        "voice-standard",
    ])
    require({"master", "published", "output_path", "output_relative_path",
             "preset", "mime_type", "engine"} <= exported_opus["data"].keys(),
            "record export-opus contract changed")
    require(exported_opus["data"]["output_relative_path"]
            == "audio/published/second-note.opus",
            "record export-opus output path changed")
    require(exported_opus["data"]["mime_type"] == "audio/ogg; codecs=opus",
            "record export-opus MIME type changed")
    require(exported_opus["data"]["engine"] == "mock",
            "record export-opus engine changed")
    require((temp_project / "audio" / "published" / "second-note.opus").exists(),
            "record export-opus did not write publication copy")
    attached_recording = run([
        "target/debug/record", "attach", "--json", str(temp_project),
        "second-note", "Second Note",
        "audio/masters/second-note.flac",
        "audio/published/second-note.opus",
        "feeds/feed.xml",
        "tag:example.invalid,2026:second-note",
        "audio/ogg; codecs=opus",
    ])
    require({"id", "title", "metadata_path", "metadata_relative_path",
             "master", "published", "feed", "entry_id", "mime_type"}
            <= attached_recording["data"].keys(),
            "record attach contract changed")
    require(attached_recording["data"]["metadata_relative_path"]
            == "audio/metadata/second-note.toml",
            "record attach metadata path changed")
    require((temp_project / "audio" / "metadata" / "second-note.toml").exists(),
            "record attach did not write metadata sidecar")
    (temp_project / "audio" / "masters" / "second-note-revised.flac").write_bytes(
        b"revised master"
    )
    (temp_project / "audio" / "published" / "second-note-revised.opus").write_bytes(
        b"revised published"
    )
    updated_recording = run([
        "target/debug/record", "update", "--json", str(temp_project),
        "second-note", "Second Note Revised",
        "audio/masters/second-note-revised.flac",
        "audio/published/second-note-revised.opus",
        "feeds/feed.xml",
        "tag:example.invalid,2026:second-note-revised",
        "audio/ogg; codecs=opus",
    ])
    require({"id", "title", "metadata_path", "metadata_relative_path",
             "master", "published", "feed", "entry_id", "mime_type"}
            <= updated_recording["data"].keys(),
            "record update contract changed")
    require(updated_recording["data"]["id"] == "second-note",
            "record update changed recording id")
    require(updated_recording["data"]["metadata_relative_path"]
            == "audio/metadata/second-note.toml",
            "record update metadata path changed")
    prepared_feed_entry = run([
        "target/debug/record", "prepare-feed-entry", "--json", str(temp_project),
        "second-note", "2026-07-19T00:00:00Z", "Second note summary",
    ])
    require({"recording_id", "title", "entry_id", "feed", "output_path",
             "output_relative_path", "published", "mime_type", "updated"}
            <= prepared_feed_entry["data"].keys(),
            "record prepare-feed-entry contract changed")
    require(prepared_feed_entry["data"]["output_relative_path"]
            == "feeds/entries/second-note.toml",
            "record prepare-feed-entry output path changed")
    require((temp_project / "feeds" / "entries" / "second-note.toml").exists(),
            "record prepare-feed-entry did not write feed entry sidecar")
    updated_feed_entry = run([
        "target/debug/record", "update-feed-entry", "--json", str(temp_project),
        "second-note", "2026-07-20T00:00:00Z", "Second note updated summary",
    ])
    require({"recording_id", "title", "entry_id", "feed", "output_path",
             "output_relative_path", "published", "mime_type", "updated"}
            <= updated_feed_entry["data"].keys(),
            "record update-feed-entry contract changed")
    require(updated_feed_entry["data"]["output_relative_path"]
            == "feeds/entries/second-note.toml",
            "record update-feed-entry output path changed")
    require(updated_feed_entry["data"]["updated"] == "2026-07-20T00:00:00Z",
            "record update-feed-entry updated value changed")
    publication_validation = run([
        "target/debug/record", "validate-publication", "--json", str(temp_project),
        "second-note",
    ])
    require("valid" in publication_validation["data"],
            "record validate-publication contract changed")
    require(publication_validation["data"]["valid"],
            "record validate-publication unexpectedly failed")
    feed_entry_validation = run([
        "target/debug/record", "validate-feed-entry", "--json", str(temp_project),
        "second-note",
    ])
    require("valid" in feed_entry_validation["data"],
            "record validate-feed-entry contract changed")
    require(feed_entry_validation["data"]["valid"],
            "record validate-feed-entry unexpectedly failed")
    generated_feed = run([
        "target/debug/record", "generate-feed", "--json", str(temp_project),
    ])
    require({"feed_path", "feed_relative_path", "entries", "updated"}
            <= generated_feed["data"].keys(),
            "record generate-feed contract changed")
    require(generated_feed["data"]["feed_relative_path"] == "feeds/feed.xml",
            "record generate-feed feed path changed")
    require(generated_feed["data"]["entries"] >= 1,
            "record generate-feed did not include entries")
    generated_feed_path = Path(generated_feed["data"]["feed_path"])
    require(generated_feed_path.exists(),
            "record generate-feed did not write feed file")
    require("<feed xmlns=\"http://www.w3.org/2005/Atom\">" in generated_feed_path.read_text(),
            "record generate-feed did not write Atom feed")
    generated_publish = run([
        "target/debug/publish", "--dry-run", "--project", str(temp_project),
        "--target", "export", "--json",
    ])
    require({"configured", "enabled", "path", "exists", "prepared_entries",
             "invalid_entries", "invalid_entry_diagnostics"} <= generated_publish["data"]["feed"].keys(),
            "publish dry-run generated feed state contract changed")
    require(generated_publish["data"]["feed"]["path"] == "feeds/feed.xml",
            "publish dry-run generated feed path changed")
    require(generated_publish["data"]["feed"]["exists"],
            "publish dry-run did not report generated feed")
    require(generated_publish["data"]["feed"]["prepared_entries"] >= 1,
            "publish dry-run did not report prepared feed entries")
    require(generated_publish["data"]["feed"]["invalid_entries"] == 0,
            "publish dry-run reported unexpected invalid feed entries")

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
