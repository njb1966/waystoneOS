use std::fs;
use std::process;
use std::process::Command;

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

fn ffmpeg_opus_available() -> bool {
    Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).contains("libopus")
        })
        .unwrap_or(false)
}

#[test]
fn validate_rejects_invalid_audio_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate",
            &repo_path("tests/fixtures/audio/invalid-path/field-note.toml"),
        ])
        .output()
        .expect("record command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("invalid_audio_path"));
}

#[test]
fn attach_creates_recording_metadata_sidecar() {
    if !ffmpeg_opus_available() {
        eprintln!("skipping record CLI publish flow because ffmpeg/libopus is unavailable");
        return;
    }

    let root = std::env::temp_dir().join(format!("waystone-record-cli-{}", process::id()));
    let project = root.join("attach-audio.wayproject");
    let metadata_path = project.join("audio/metadata/field-note.toml");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
    fs::create_dir_all(project.join("audio/published")).expect("published directory");
    fs::create_dir_all(project.join("feeds")).expect("feeds directory");
    fs::write(
        project.join("project.toml"),
        r#"[waystone]
schema = 1
created_by = "WaystoneOS"

[project]
id = "attach-audio"
name = "Attach Audio"
type = "audio-series"
language = "en"

[content]
root = "content"
index = "index.gmi"

[audio]
masters = "audio/masters"
published = "audio/published"
metadata = "audio/metadata"

[feed]
enabled = true
type = "atom"
path = "feeds/feed.xml"
title = "Attach Audio"
"#,
    )
    .expect("project manifest");

    let capture_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "capture",
            "--json",
            project.to_str().expect("project path"),
            "audio/masters/field-note.wav",
            "1",
            "lavfi",
            "anullsrc=r=48000:cl=mono",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(capture_output.status.code(), Some(0));
    let capture_stdout = String::from_utf8_lossy(&capture_output.stdout);
    assert!(capture_stdout.contains("\"output_relative_path\":\"audio/masters/field-note.wav\""));
    assert!(capture_stdout.contains("\"duration_seconds\":1"));
    assert!(capture_stdout.contains("\"channels\":1"));
    assert!(capture_stdout.contains("\"sample_rate\":48000"));
    assert!(capture_stdout.contains("\"format\":\"wav\""));
    assert!(capture_stdout.contains("\"engine\":\"ffmpeg\""));
    let captured_master = fs::read(project.join("audio/masters/field-note.wav"))
        .expect("captured master should exist");
    assert!(captured_master.starts_with(b"RIFF"));

    let export_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "export-opus",
            "--json",
            project.to_str().expect("project path"),
            "audio/masters/field-note.wav",
            "audio/published/field-note.opus",
            "voice-standard",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(export_output.status.code(), Some(0));
    let export_stdout = String::from_utf8_lossy(&export_output.stdout);
    assert!(export_stdout.contains("\"output_relative_path\":\"audio/published/field-note.opus\""));
    assert!(export_stdout.contains("\"mime_type\":\"audio/ogg; codecs=opus\""));
    assert!(export_stdout.contains("\"engine\":\"ffmpeg\""));
    assert!(project.join("audio/published/field-note.opus").is_file());

    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "attach",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Field Note",
            "audio/masters/field-note.wav",
            "audio/published/field-note.opus",
            "feeds/feed.xml",
            "tag:example.invalid,2026:field-note",
            "audio/ogg; codecs=opus",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"metadata_relative_path\":\"audio/metadata/field-note.toml\""));
    assert!(metadata_path.is_file());

    let metadata = fs::read_to_string(&metadata_path).expect("metadata sidecar");
    assert!(metadata.contains("[publication]"));
    assert!(metadata.contains("feed = \"feeds/feed.xml\""));
    assert!(metadata.contains("mime_type = \"audio/ogg; codecs=opus\""));

    let feed_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "prepare-feed-entry",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "2026-07-19T00:00:00Z",
            "Field note summary",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(feed_output.status.code(), Some(0));
    let feed_stdout = String::from_utf8_lossy(&feed_output.stdout);
    assert!(feed_stdout.contains("\"output_relative_path\":\"feeds/entries/field-note.toml\""));
    let feed_entry_path = project.join("feeds/entries/field-note.toml");
    assert!(feed_entry_path.is_file());

    let feed_entry = fs::read_to_string(&feed_entry_path).expect("feed entry sidecar");
    assert!(feed_entry.contains("[entry]"));
    assert!(feed_entry.contains("recording_metadata = \"audio/metadata/field-note.toml\""));
    assert!(feed_entry.contains("[enclosure]"));
    assert!(feed_entry.contains("path = \"audio/published/field-note.opus\""));

    let update_feed_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "update-feed-entry",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "2026-07-20T00:00:00Z",
            "Field note updated summary",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(update_feed_output.status.code(), Some(0));
    let update_feed_stdout = String::from_utf8_lossy(&update_feed_output.stdout);
    assert!(
        update_feed_stdout.contains("\"output_relative_path\":\"feeds/entries/field-note.toml\"")
    );
    assert!(update_feed_stdout.contains("\"updated\":\"2026-07-20T00:00:00Z\""));
    let updated_feed_entry = fs::read_to_string(&feed_entry_path).expect("updated feed entry");
    assert!(updated_feed_entry.contains("updated = \"2026-07-20T00:00:00Z\""));
    assert!(updated_feed_entry.contains("summary = \"Field note updated summary\""));

    let publication_validation = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate-publication",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(publication_validation.status.code(), Some(0));
    let publication_stdout = String::from_utf8_lossy(&publication_validation.stdout);
    assert!(publication_stdout.contains("\"valid\":true"));

    let feed_validation = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate-feed-entry",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(feed_validation.status.code(), Some(0));
    let feed_validation_stdout = String::from_utf8_lossy(&feed_validation.stdout);
    assert!(feed_validation_stdout.contains("\"valid\":true"));

    fs::write(
        project.join("feeds/feed.xml"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Existing Feed</title>
  <id>waystone:feed:feeds/feed.xml</id>
  <updated>2026-07-18T00:00:00Z</updated>
  <entry>
    <id>tag:example.invalid,2026:field-note</id>
    <title>Stale Field Note</title>
    <updated>2026-07-18T00:00:00Z</updated>
    <summary>Stale summary</summary>
  </entry>
  <entry>
    <id>tag:example.invalid,2026:manual</id>
    <title>Manual Entry</title>
    <updated>2026-07-21T00:00:00Z</updated>
    <summary>Preserved manual entry</summary>
  </entry>
</feed>
"#,
    )
    .expect("existing feed XML");

    let generate_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "generate-feed",
            "--json",
            project.to_str().expect("project path"),
        ])
        .output()
        .expect("record command should run");

    assert_eq!(generate_output.status.code(), Some(0));
    let generate_stdout = String::from_utf8_lossy(&generate_output.stdout);
    assert!(generate_stdout.contains("\"feed_relative_path\":\"feeds/feed.xml\""));
    assert!(generate_stdout.contains("\"entries\":2"));
    assert!(generate_stdout.contains("\"updated\":\"2026-07-21T00:00:00Z\""));
    let generated_feed = fs::read_to_string(project.join("feeds/feed.xml")).expect("feed xml");
    assert!(generated_feed.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"));
    assert!(generated_feed.contains("<title>Attach Audio</title>"));
    assert!(generated_feed.contains("<id>tag:example.invalid,2026:field-note</id>"));
    assert!(!generated_feed.contains("Stale Field Note"));
    assert!(generated_feed.contains("<id>tag:example.invalid,2026:manual</id>"));
    assert!(
        generated_feed.contains("<link rel=\"enclosure\" href=\"audio/published/field-note.opus\"")
    );

    fs::write(
        project.join("feeds/entries/duplicate.toml"),
        r#"[entry]
id = "tag:example.invalid,2026:field-note"
title = "Duplicate"
updated = "2026-07-19T00:00:00Z"
summary = "Duplicate summary"
feed = "feeds/feed.xml"
recording = "field-note"
recording_metadata = "audio/metadata/field-note.toml"

[enclosure]
path = "audio/published/field-note.opus"
mime_type = "audio/ogg; codecs=opus"
"#,
    )
    .expect("duplicate feed entry sidecar");

    let duplicate_feed_validation = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate-feed-entry",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(duplicate_feed_validation.status.code(), Some(3));
    let duplicate_feed_validation_stdout =
        String::from_utf8_lossy(&duplicate_feed_validation.stdout);
    assert!(duplicate_feed_validation_stdout.contains("duplicate_feed_entry_id"));

    let duplicate_generate_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "generate-feed",
            "--json",
            project.to_str().expect("project path"),
        ])
        .output()
        .expect("record command should run");

    assert_eq!(duplicate_generate_output.status.code(), Some(1));
    let duplicate_generate_stdout = String::from_utf8_lossy(&duplicate_generate_output.stdout);
    assert!(duplicate_generate_stdout.contains("duplicate_feed_entry_id"));

    let duplicate_feed_output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "prepare-feed-entry",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "2026-07-19T00:00:00Z",
            "Field note summary",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(duplicate_feed_output.status.code(), Some(1));
    let duplicate_feed_stdout = String::from_utf8_lossy(&duplicate_feed_output.stdout);
    assert!(duplicate_feed_stdout.contains("already exists"));

    let duplicate = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "attach",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Field Note",
            "audio/masters/field-note.wav",
            "audio/published/field-note.opus",
            "feeds/feed.xml",
            "tag:example.invalid,2026:field-note",
            "audio/ogg; codecs=opus",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(duplicate.status.code(), Some(1));
    let duplicate_stdout = String::from_utf8_lossy(&duplicate.stdout);
    assert!(duplicate_stdout.contains("already exists"));

    let duplicate_export = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "export-opus",
            "--json",
            project.to_str().expect("project path"),
            "audio/masters/field-note.wav",
            "audio/published/field-note.opus",
            "voice-standard",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(duplicate_export.status.code(), Some(1));
    let duplicate_export_stdout = String::from_utf8_lossy(&duplicate_export.stdout);
    assert!(duplicate_export_stdout.contains("already exists"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn update_replaces_recording_metadata_sidecar() {
    let root = std::env::temp_dir().join(format!("waystone-record-cli-update-{}", process::id()));
    let project = root.join("update-audio.wayproject");
    let metadata_path = project.join("audio/metadata/field-note.toml");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
    fs::create_dir_all(project.join("audio/published")).expect("published directory");
    fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
    fs::create_dir_all(project.join("feeds")).expect("feeds directory");
    fs::write(
        project.join("project.toml"),
        r#"[waystone]
schema = 1
created_by = "WaystoneOS"

[project]
id = "update-audio"
name = "Update Audio"
type = "audio-series"
language = "en"

[content]
root = "content"
index = "index.gmi"

[audio]
masters = "audio/masters"
published = "audio/published"
metadata = "audio/metadata"

[feed]
enabled = true
type = "atom"
path = "feeds/feed.xml"
title = "Update Audio"
"#,
    )
    .expect("project manifest");
    fs::write(project.join("audio/masters/original.flac"), b"master").expect("master file");
    fs::write(
        project.join("audio/masters/revised.flac"),
        b"revised master",
    )
    .expect("revised master file");
    fs::write(project.join("audio/published/original.opus"), b"published").expect("published file");
    fs::write(
        project.join("audio/published/revised.opus"),
        b"revised published",
    )
    .expect("revised published file");
    fs::write(
        &metadata_path,
        r#"[recording]
id = "field-note"
title = "Original Field Note"
master = "audio/masters/original.flac"
published = "audio/published/original.opus"
duration_seconds = 42

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:original"
mime_type = "audio/ogg; codecs=opus"
"#,
    )
    .expect("recording metadata");

    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "update",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Revised Field Note",
            "audio/masters/revised.flac",
            "audio/published/revised.opus",
            "feeds/feed.xml",
            "tag:example.invalid,2026:revised",
            "audio/ogg; codecs=opus",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":\"field-note\""));
    assert!(stdout.contains("\"title\":\"Revised Field Note\""));
    assert!(stdout.contains("\"metadata_relative_path\":\"audio/metadata/field-note.toml\""));

    let metadata = fs::read_to_string(&metadata_path).expect("metadata sidecar");
    assert!(metadata.contains("id = \"field-note\""));
    assert!(metadata.contains("title = \"Revised Field Note\""));
    assert!(metadata.contains("master = \"audio/masters/revised.flac\""));
    assert!(metadata.contains("published = \"audio/published/revised.opus\""));
    assert!(metadata.contains("duration_seconds = 42"));
    assert!(metadata.contains("entry_id = \"tag:example.invalid,2026:revised\""));

    let validation = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate-publication",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(validation.status.code(), Some(0));
    let validation_stdout = String::from_utf8_lossy(&validation.stdout);
    assert!(validation_stdout.contains("\"valid\":true"));

    let escape = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "update",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Revised Field Note",
            "../outside.flac",
            "audio/published/revised.opus",
            "feeds/feed.xml",
            "tag:example.invalid,2026:revised",
            "audio/ogg; codecs=opus",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(escape.status.code(), Some(1));
    let escape_stdout = String::from_utf8_lossy(&escape.stdout);
    assert!(escape_stdout.contains("invalid audio path"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn validate_publication_rejects_missing_published_audio() {
    let root = std::env::temp_dir().join(format!(
        "waystone-record-cli-missing-published-{}",
        process::id()
    ));
    let project = root.join("missing-published.wayproject");
    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
    fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
    fs::write(
        project.join("project.toml"),
        r#"[waystone]
schema = 1
created_by = "WaystoneOS"

[project]
id = "missing-published"
name = "Missing Published"
type = "audio-series"

[content]
root = "content"
index = "index.gmi"

[audio]
masters = "audio/masters"
published = "audio/published"
metadata = "audio/metadata"
"#,
    )
    .expect("project manifest");
    fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
    fs::write(
        project.join("audio/metadata/note.toml"),
        r#"[recording]
id = "note"
title = "Note"
master = "audio/masters/note.flac"
published = "audio/published/missing.opus"

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:note"
mime_type = "audio/ogg; codecs=opus"
"#,
    )
    .expect("recording metadata");

    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "validate-publication",
            "--json",
            project.to_str().expect("project path"),
            "note",
        ])
        .output()
        .expect("record command should run");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("missing_published_audio"));

    let _ = fs::remove_dir_all(root);
}
