use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(relative: &str) -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .display()
        .to_string()
}

fn unique_temp_project_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "waystone-publish-cli-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be available")
            .as_nanos()
    ))
}

fn copy_directory(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("destination directory should be created");
    for entry in std::fs::read_dir(from).expect("source directory should be readable") {
        let entry = entry.expect("source entry should be readable");
        let source = entry.path();
        let destination = to.join(entry.file_name());
        if source.is_dir() {
            copy_directory(&source, &destination);
        } else {
            std::fs::copy(&source, &destination).expect("file should be copied");
        }
    }
}

#[test]
fn dry_run_json_reports_resolved_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--dry-run",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"blocked\":false"));
    assert!(stdout.contains("\"host_resolution\""));
    assert!(stdout.contains("\"identity_resolution\""));
}

#[test]
fn dry_run_json_reports_feed_state() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--dry-run",
            "--project",
            &repo_path("examples/projects/audio-capsule.wayproject"),
            "--target",
            "export",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"feed\":{"));
    assert!(stdout.contains("\"configured\":true"));
    assert!(stdout.contains("\"enabled\":true"));
    assert!(stdout.contains("\"path\":\"feeds/feed.xml\""));
    assert!(stdout.contains("\"exists\":true"));
    assert!(stdout.contains("\"prepared_entries\":0"));
    assert!(stdout.contains("\"invalid_entries\":0"));
    assert!(stdout.contains("\"invalid_entry_diagnostics\":[]"));
}

#[test]
fn dry_run_json_reports_invalid_feed_entry_diagnostics() {
    let project_root = unique_temp_project_root("feed-diagnostics");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/audio-capsule.wayproject")),
        &project_root,
    );
    std::fs::create_dir_all(project_root.join("feeds/entries")).expect("feed entries directory");
    std::fs::write(
        project_root.join("feeds/entries/broken.toml"),
        r#"[entry]
id = "tag:example.invalid,2026:broken"
title = "Broken"
updated = "2026-07-20T00:00:00Z"
summary = "Broken summary"
feed = "feeds/feed.xml"
recording = "missing"

[enclosure]
path = "audio/published/missing.opus"
mime_type = "audio/ogg; codecs=opus"
"#,
    )
    .expect("broken feed entry");

    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--dry-run",
            "--project",
            project_root
                .to_str()
                .expect("temp project path should be utf-8"),
            "--target",
            "export",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"invalid_entries\":1"));
    assert!(stdout.contains("\"invalid_entry_diagnostics\":["));
    assert!(stdout.contains("\"path\":\"feeds/entries/broken.toml\""));
    assert!(stdout.contains("entry.recording_metadata"));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn planned_history_json_reports_inspectable_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--planned-history",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--date",
            "2026-07-18T00:00:00Z",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"transfer_result\":\"planned\""));
    assert!(stdout.contains("\"verification_result\":\"not-run\""));
    assert!(stdout.contains("\"files\":["));
    assert!(stdout.contains("\"action\":\"planned-upload\""));
    assert!(stdout.contains("[publication]\\n"));
    assert!(stdout.contains("planned-upload"));
    assert!(stdout.contains("content/index.gmi"));
}

#[test]
fn save_planned_history_preview_writes_under_project_history_previews() {
    let project_root = unique_temp_project_root("history-preview");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-planned-history-preview",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-19T00:00:00Z",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"output_path\""));
    assert!(stdout.contains("history/previews"));
    assert!(stdout.contains("2026-07-19T00-00-00Z-production-planned.toml"));

    let saved_path = project_root
        .join("history")
        .join("previews")
        .join("2026-07-19T00-00-00Z-production-planned.toml");
    assert!(saved_path.exists());
    assert!(std::fs::read_to_string(saved_path)
        .expect("saved preview should be readable")
        .contains("transfer_result = \"planned\""));
}

#[test]
fn list_planned_history_previews_reports_saved_previews() {
    let project_root = unique_temp_project_root("history-preview-list");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let save_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-planned-history-preview",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-19T00:00:00Z",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");
    assert!(save_output.status.success());

    let list_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--list-planned-history-previews",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(list_output.status.success());
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("\"previews\":["));
    assert!(stdout.contains("\"filename\":\"2026-07-19T00-00-00Z-production-planned.toml\""));
    assert!(stdout.contains("\"modified_unix\":"));
    assert!(stdout.contains("\"size_bytes\":"));
    assert!(stdout.contains("history/previews"));
}

#[test]
fn read_planned_history_preview_reports_saved_preview_detail() {
    let project_root = unique_temp_project_root("history-preview-read");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let save_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-planned-history-preview",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-19T00:00:00Z",
            "--hosts",
            &repo_path("examples/connections/hosts"),
            "--identities",
            &repo_path("examples/connections/identities"),
            "--json",
        ])
        .output()
        .expect("publish command should run");
    assert!(save_output.status.success());

    let saved_path = project_root
        .join("history")
        .join("previews")
        .join("2026-07-19T00-00-00Z-production-planned.toml");
    let read_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--read-planned-history-preview",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--preview",
            saved_path.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(read_output.status.success());
    let stdout = String::from_utf8_lossy(&read_output.stdout);
    assert!(stdout.contains("\"record_toml\":\"[publication]\\n"));
    assert!(stdout.contains("\"filename\":\"2026-07-19T00-00-00Z-production-planned.toml\""));
    assert!(stdout.contains("transfer_result = \\\"planned\\\""));
}

#[test]
fn read_planned_history_preview_rejects_outside_project_previews() {
    let project_root = unique_temp_project_root("history-preview-read-reject");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );
    std::fs::create_dir_all(project_root.join("history").join("previews"))
        .expect("preview directory should be created");
    let outside = project_root.join("outside.toml");
    std::fs::write(&outside, "[publication]\n").expect("outside file should be written");

    let read_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--read-planned-history-preview",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--preview",
            outside.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(!read_output.status.success());
    let stdout = String::from_utf8_lossy(&read_output.stdout);
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("outside project preview directory"));
}
