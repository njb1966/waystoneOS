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
    fs::write(project.join("audio/masters/field-note.flac"), b"master").expect("master file");
    fs::write(
        project.join("audio/published/field-note.opus"),
        b"published",
    )
    .expect("published file");

    let output = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "attach",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Field Note",
            "audio/masters/field-note.flac",
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

    let duplicate = Command::new(env!("CARGO_BIN_EXE_record"))
        .args([
            "attach",
            "--json",
            project.to_str().expect("project path"),
            "field-note",
            "Field Note",
            "audio/masters/field-note.flac",
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

    let _ = fs::remove_dir_all(root);
}
