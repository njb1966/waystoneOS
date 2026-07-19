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
