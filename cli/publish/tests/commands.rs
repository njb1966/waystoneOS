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
fn dry_run_json_reports_remote_state_comparison() {
    let temp_root = unique_temp_project_root("remote-state");
    std::fs::create_dir_all(&temp_root).expect("temp root should be created");
    let remote_state = temp_root.join("remote-state.txt");
    std::fs::write(&remote_state, "content/index.gmi\nstale.gmi\n")
        .expect("remote state should be written");

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
            "--remote-state",
            remote_state.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"comparison\":{\"configured\":true"));
    assert!(stdout.contains("\"remote_paths\":2"));
    assert!(stdout.contains("\"upload\":[]"));
    assert!(stdout.contains("\"update\":[]"));
    assert!(stdout.contains("\"delete\":[\"stale.gmi\"]"));
    assert!(stdout.contains("\"skip\":[\"content/index.gmi\"]"));

    let _ = std::fs::remove_dir_all(temp_root);
}

#[test]
fn export_remote_state_json_reports_publishable_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--export-remote-state",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"project\":\"ssh-capsule\""));
    assert!(stdout.contains("\"target\":\"production\""));
    assert!(stdout.contains("\"path_count\":1"));
    assert!(stdout.contains("\"paths\":[\"content/index.gmi\"]"));
    assert!(stdout.contains("\"manifest\":\"content/index.gmi\\n\""));
}

#[test]
fn export_and_inspect_remote_state_file() {
    let temp_root = unique_temp_project_root("remote-state-export");
    std::fs::create_dir_all(&temp_root).expect("temp root should be created");
    let remote_state = temp_root.join("remote-state.txt");

    let export = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--export-remote-state",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--output",
            remote_state.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(export.status.success());
    assert_eq!(
        std::fs::read_to_string(&remote_state).expect("remote state should be readable"),
        "content/index.gmi\n"
    );

    let inspect = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--inspect-remote-state",
            "--remote-state",
            remote_state.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(inspect.status.success());
    let stdout = String::from_utf8_lossy(&inspect.stdout);
    assert!(stdout.contains("\"source\":"));
    assert!(stdout.contains("\"path_count\":1"));
    assert!(stdout.contains("\"paths\":[\"content/index.gmi\"]"));

    let overwrite = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--export-remote-state",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--output",
            remote_state.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(!overwrite.status.success());
    let stdout = String::from_utf8_lossy(&overwrite.stdout);
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("File exists") || stdout.contains("exists"));

    let _ = std::fs::remove_dir_all(temp_root);
}

#[test]
fn validate_json_reports_ready_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--validate",
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
    assert!(stdout.contains("\"valid\":true"));
    assert!(stdout.contains("\"blocked\":false"));
    assert!(stdout.contains("\"errors\":[]"));
    assert!(stdout.contains("\"code\":\"confirmation_required\""));
}

#[test]
fn validate_json_reports_blocked_target_without_metadata_roots() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--validate",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"valid\":false"));
    assert!(stdout.contains("\"blocked\":true"));
    assert!(stdout.contains("\"code\":\"host_missing\""));
    assert!(stdout.contains("\"code\":\"identity_missing\""));
}

#[test]
fn transfer_intent_json_reports_ready_removable_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--transfer-intent",
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
    assert!(stdout.contains("\"project\":\"audio-capsule\""));
    assert!(stdout.contains("\"target\":\"export\""));
    assert!(stdout.contains("\"method\":\"removable\""));
    assert!(stdout.contains("\"execution_ready\":true"));
    assert!(stdout.contains("\"blocked_reasons\":[]"));
    assert!(stdout.contains("\"confirmations\":[]"));
    assert!(stdout.contains("\"changes\":{"));
    assert!(stdout.contains("\"upload\":["));
    assert!(stdout.contains("\"history\":{\"completed_directory\":\""));
}

#[test]
fn transfer_intent_json_reports_blocked_target_without_metadata_roots() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--transfer-intent",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"execution_ready\":false"));
    assert!(stdout.contains("\"blocked_reasons\":["));
    assert!(stdout.contains("\"code\":\"host_missing\""));
    assert!(stdout.contains("\"code\":\"identity_missing\""));
    assert!(stdout.contains("\"host_resolution\":{\"id\":\"offgridholdout\""));
    assert!(stdout.contains("\"identity_resolution\":{\"id\":\"nick-pub\""));
}

#[test]
fn prepare_removable_execution_json_reports_operation_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--prepare-removable-execution",
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
    assert!(stdout.contains("\"project\":\"audio-capsule\""));
    assert!(stdout.contains("\"target\":\"export\""));
    assert!(stdout.contains("\"method\":\"removable\""));
    assert!(stdout.contains("\"destination_root\":\""));
    assert!(stdout.contains("publish/export"));
    assert!(stdout.contains("\"execution_ready\":true"));
    assert!(stdout.contains("\"blocked_reasons\":[]"));
    assert!(stdout.contains("\"operations\":{"));
    assert!(stdout.contains("\"upload\":["));
    assert!(stdout.contains("\"project_path\":\"content/index.gmi\""));
    assert!(stdout.contains("\"source_path\":\""));
    assert!(stdout.contains("\"destination_path\":\""));
    assert!(stdout.contains("\"delete\":[]"));
    assert!(stdout.contains("\"history\":{\"completed_directory\":\""));
}

#[test]
fn prepare_removable_execution_json_blocks_unsupported_methods() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--prepare-removable-execution",
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
    assert!(stdout.contains("\"execution_ready\":false"));
    assert!(stdout.contains("\"code\":\"unsupported_executor_method\""));
}

#[test]
fn execute_removable_json_copies_files_and_writes_history() {
    let project_root = unique_temp_project_root("execute-removable");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/audio-capsule.wayproject")),
        &project_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--execute-removable",
            "--project",
            project_root
                .to_str()
                .expect("temp project path should be utf-8"),
            "--target",
            "export",
            "--date",
            "2026-07-21T00:00:00Z",
            "--confirm-transfer",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(output.status.success());
    assert!(project_root
        .join("publish/export/content/index.gmi")
        .is_file());
    assert!(project_root
        .join("publish/export/audio/published/field-note.opus")
        .is_file());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"transfer_result\":\"completed\""));
    assert!(stdout.contains("\"verification_result\":\"not-run\""));
    assert!(stdout.contains("\"files\":["));
    assert!(stdout.contains("\"action\":\"upload\""));
    assert!(stdout.contains("\"result\":\"copied\""));
    assert!(stdout.contains("\"bytes\":"));
    assert!(stdout.contains("\"history\":{\"completed_path\":\""));
    assert!(stdout.contains("history/completed"));
    assert!(stdout.contains("copied-upload"));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn execute_removable_json_reports_partial_result_and_writes_history() {
    let project_root = unique_temp_project_root("execute-removable-partial");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/audio-capsule.wayproject")),
        &project_root,
    );
    let content_path_collision = project_root.join("publish/export/content");
    std::fs::create_dir_all(
        content_path_collision
            .parent()
            .expect("collision path has parent"),
    )
    .expect("collision parent should be created");
    std::fs::write(&content_path_collision, "not a directory")
        .expect("collision file should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--execute-removable",
            "--project",
            project_root
                .to_str()
                .expect("temp project path should be utf-8"),
            "--target",
            "export",
            "--date",
            "2026-07-21T00:00:00Z",
            "--confirm-transfer",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(!output.status.success());
    assert!(project_root
        .join("publish/export/audio/published/field-note.opus")
        .is_file());
    assert!(!project_root
        .join("publish/export/content/index.gmi")
        .exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":\"ok\""));
    assert!(stdout.contains("\"transfer_result\":\"partial\""));
    assert!(stdout.contains("\"project_path\":\"content/index.gmi\""));
    assert!(stdout.contains("\"result\":\"failed\""));
    assert!(stdout.contains("\"error\":\""));
    assert!(stdout.contains("\"history\":{\"completed_path\":\""));
    assert!(stdout.contains("failed-upload"));

    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn execute_removable_json_requires_confirm_transfer() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--execute-removable",
            "--project",
            &repo_path("examples/projects/audio-capsule.wayproject"),
            "--target",
            "export",
            "--date",
            "2026-07-21T00:00:00Z",
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("confirm-transfer"));
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
fn completed_history_json_reports_inspectable_result_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--completed-history",
            "--project",
            &repo_path("examples/projects/ssh-capsule.wayproject"),
            "--target",
            "production",
            "--date",
            "2026-07-20T00:00:00Z",
            "--transfer-result",
            "completed",
            "--verification-result",
            "passed",
            "--rollback-available",
            "false",
            "--rollback-notes",
            "No rollback snapshot recorded",
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
    assert!(stdout.contains("\"transfer_result\":\"completed\""));
    assert!(stdout.contains("\"verification_result\":\"passed\""));
    assert!(stdout.contains("\"files\":["));
    assert!(stdout.contains("\"action\":\"planned-upload\""));
    assert!(stdout.contains("[publication]\\n"));
    assert!(stdout.contains("transfer_result = \\\"completed\\\""));
    assert!(stdout.contains("verification_result = \\\"passed\\\""));
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
fn save_completed_history_writes_under_project_history_completed() {
    let project_root = unique_temp_project_root("history-completed");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-20T00:00:00Z",
            "--transfer-result",
            "completed",
            "--verification-result",
            "passed",
            "--rollback-available",
            "false",
            "--rollback-notes",
            "No rollback snapshot recorded",
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
    assert!(stdout.contains("history/completed"));
    assert!(stdout.contains("2026-07-20T00-00-00Z-production-completed.toml"));

    let saved_path = project_root
        .join("history")
        .join("completed")
        .join("2026-07-20T00-00-00Z-production-completed.toml");
    assert!(saved_path.exists());
    let record = std::fs::read_to_string(saved_path).expect("saved record should be readable");
    assert!(record.contains("transfer_result = \"completed\""));
    assert!(record.contains("verification_result = \"passed\""));
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
fn list_completed_history_reports_saved_records() {
    let project_root = unique_temp_project_root("history-completed-list");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let save_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-20T00:00:00Z",
            "--transfer-result",
            "completed",
            "--verification-result",
            "passed",
            "--rollback-available",
            "false",
            "--rollback-notes",
            "No rollback snapshot recorded",
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
            "--list-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(list_output.status.success());
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("\"records\":["));
    assert!(stdout.contains("\"filename\":\"2026-07-20T00-00-00Z-production-completed.toml\""));
    assert!(stdout.contains("\"modified_unix\":"));
    assert!(stdout.contains("\"size_bytes\":"));
    assert!(stdout.contains("history/completed"));
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
fn read_completed_history_reports_saved_record_detail() {
    let project_root = unique_temp_project_root("history-completed-read");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );

    let save_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--save-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--target",
            "production",
            "--date",
            "2026-07-20T00:00:00Z",
            "--transfer-result",
            "completed",
            "--verification-result",
            "passed",
            "--rollback-available",
            "false",
            "--rollback-notes",
            "No rollback snapshot recorded",
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
        .join("completed")
        .join("2026-07-20T00-00-00Z-production-completed.toml");
    let read_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--read-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--record",
            saved_path.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(read_output.status.success());
    let stdout = String::from_utf8_lossy(&read_output.stdout);
    assert!(stdout.contains("\"record_toml\":\"[publication]\\n"));
    assert!(stdout.contains("\"filename\":\"2026-07-20T00-00-00Z-production-completed.toml\""));
    assert!(stdout.contains("transfer_result = \\\"completed\\\""));
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

#[test]
fn read_completed_history_rejects_outside_project_completed_history() {
    let project_root = unique_temp_project_root("history-completed-read-reject");
    copy_directory(
        std::path::Path::new(&repo_path("examples/projects/ssh-capsule.wayproject")),
        &project_root,
    );
    std::fs::create_dir_all(project_root.join("history").join("completed"))
        .expect("completed history directory should be created");
    let outside = project_root.join("outside.toml");
    std::fs::write(&outside, "[publication]\n").expect("outside file should be written");

    let read_output = Command::new(env!("CARGO_BIN_EXE_publish"))
        .args([
            "--read-completed-history",
            "--project",
            project_root.to_str().expect("temp path should be utf-8"),
            "--record",
            outside.to_str().expect("temp path should be utf-8"),
            "--json",
        ])
        .output()
        .expect("publish command should run");

    assert!(!read_output.status.success());
    let stdout = String::from_utf8_lossy(&read_output.stdout);
    assert!(stdout.contains("\"status\":\"error\""));
    assert!(stdout.contains("outside project completed history directory"));
}
