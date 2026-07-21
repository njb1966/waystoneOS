use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use waystone_publish_plan::PublishDryRun;

pub const HISTORY_SCHEMA: u32 = 1;

#[derive(Debug, Clone)]
pub struct PublicationHistoryRecord {
    pub schema: u32,
    pub date: String,
    pub project_id: String,
    pub target: String,
    pub method: String,
    pub destination: Option<String>,
    pub transfer_result: String,
    pub verification_result: String,
    pub files: Vec<PublicationFile>,
    pub rollback: RollbackInfo,
}

#[derive(Debug, Clone)]
pub struct PublicationFile {
    pub path: String,
    pub action: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RollbackInfo {
    pub available: bool,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct CompletedHistoryOptions {
    pub transfer_result: String,
    pub verification_result: String,
    pub rollback_available: bool,
    pub rollback_notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlannedHistoryPreviewEntry {
    pub path: PathBuf,
    pub filename: String,
    pub modified_unix: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlannedHistoryPreviewDetail {
    pub entry: PlannedHistoryPreviewEntry,
    pub record_toml: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompletedHistoryEntry {
    pub path: PathBuf,
    pub filename: String,
    pub modified_unix: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompletedHistoryDetail {
    pub entry: CompletedHistoryEntry,
    pub record_toml: String,
}

impl PublicationHistoryRecord {
    pub fn planned_from_dry_run(plan: &PublishDryRun, date: impl Into<String>) -> Self {
        let files = plan
            .upload
            .iter()
            .map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-upload".to_string(),
                sha256: None,
            })
            .chain(plan.update.iter().map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-update".to_string(),
                sha256: None,
            }))
            .chain(plan.delete.iter().map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-delete".to_string(),
                sha256: None,
            }))
            .collect();

        Self {
            schema: HISTORY_SCHEMA,
            date: date.into(),
            project_id: plan.project_id.clone(),
            target: plan.target.clone(),
            method: plan.method.clone(),
            destination: plan.destination.clone(),
            transfer_result: "planned".to_string(),
            verification_result: "not-run".to_string(),
            files,
            rollback: RollbackInfo {
                available: false,
                notes: "Dry-run only; no remote state changed".to_string(),
            },
        }
    }

    pub fn completed_from_dry_run(
        plan: &PublishDryRun,
        date: impl Into<String>,
        options: CompletedHistoryOptions,
    ) -> Self {
        let mut record = Self::planned_from_dry_run(plan, date);
        record.transfer_result = options.transfer_result;
        record.verification_result = options.verification_result;
        record.rollback = RollbackInfo {
            available: options.rollback_available,
            notes: options.rollback_notes,
        };
        record
    }

    pub fn to_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("[publication]\n");
        output.push_str(&format!("schema = {}\n", self.schema));
        output.push_str(&format!("date = \"{}\"\n", toml_escape(&self.date)));
        output.push_str(&format!(
            "project_id = \"{}\"\n",
            toml_escape(&self.project_id)
        ));
        output.push_str(&format!("target = \"{}\"\n", toml_escape(&self.target)));
        output.push_str(&format!("method = \"{}\"\n", toml_escape(&self.method)));
        if let Some(destination) = &self.destination {
            output.push_str(&format!("destination = \"{}\"\n", toml_escape(destination)));
        }
        output.push_str(&format!(
            "transfer_result = \"{}\"\n",
            toml_escape(&self.transfer_result)
        ));
        output.push_str(&format!(
            "verification_result = \"{}\"\n",
            toml_escape(&self.verification_result)
        ));
        output.push('\n');

        for file in &self.files {
            output.push_str("[[files]]\n");
            output.push_str(&format!("path = \"{}\"\n", toml_escape(&file.path)));
            output.push_str(&format!("action = \"{}\"\n", toml_escape(&file.action)));
            if let Some(hash) = &file.sha256 {
                output.push_str(&format!("sha256 = \"{}\"\n", toml_escape(hash)));
            }
            output.push('\n');
        }

        output.push_str("[rollback]\n");
        output.push_str(&format!("available = {}\n", self.rollback.available));
        output.push_str(&format!(
            "notes = \"{}\"\n",
            toml_escape(&self.rollback.notes)
        ));

        output
    }
}

pub fn write_planned_history_preview(
    project_root: impl AsRef<Path>,
    record: &PublicationHistoryRecord,
) -> io::Result<PathBuf> {
    let preview_dir = project_root.as_ref().join("history").join("previews");
    fs::create_dir_all(&preview_dir)?;

    let filename = format!(
        "{}-{}-planned.toml",
        safe_filename_segment(&record.date),
        safe_filename_segment(&record.target)
    );
    let output_path = preview_dir.join(filename);
    let temp_path = output_path.with_extension("toml.tmp");
    fs::write(&temp_path, record.to_toml())?;
    fs::rename(&temp_path, &output_path)?;
    Ok(output_path)
}

pub fn write_completed_history_record(
    project_root: impl AsRef<Path>,
    record: &PublicationHistoryRecord,
) -> io::Result<PathBuf> {
    let history_dir = project_root.as_ref().join("history").join("completed");
    fs::create_dir_all(&history_dir)?;

    let filename = format!(
        "{}-{}-completed.toml",
        safe_filename_segment(&record.date),
        safe_filename_segment(&record.target)
    );
    let output_path = history_dir.join(filename);
    let temp_path = output_path.with_extension("toml.tmp");
    fs::write(&temp_path, record.to_toml())?;
    fs::rename(&temp_path, &output_path)?;
    Ok(output_path)
}

pub fn list_planned_history_previews(
    project_root: impl AsRef<Path>,
) -> io::Result<Vec<PlannedHistoryPreviewEntry>> {
    let preview_dir = project_root.as_ref().join("history").join("previews");
    if !preview_dir.exists() {
        return Ok(Vec::new());
    }

    let mut previews = Vec::new();
    for entry in fs::read_dir(preview_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if !metadata.is_file() || path.extension().and_then(|value| value.to_str()) != Some("toml")
        {
            continue;
        }

        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        previews.push(PlannedHistoryPreviewEntry {
            filename: entry.file_name().to_string_lossy().to_string(),
            path,
            modified_unix,
            size_bytes: metadata.len(),
        });
    }

    previews.sort_by(|left, right| {
        right
            .modified_unix
            .cmp(&left.modified_unix)
            .then_with(|| left.filename.cmp(&right.filename))
    });
    Ok(previews)
}

pub fn list_completed_history_records(
    project_root: impl AsRef<Path>,
) -> io::Result<Vec<CompletedHistoryEntry>> {
    let history_dir = project_root.as_ref().join("history").join("completed");
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    for entry in fs::read_dir(history_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if !metadata.is_file() || path.extension().and_then(|value| value.to_str()) != Some("toml")
        {
            continue;
        }

        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        records.push(CompletedHistoryEntry {
            filename: entry.file_name().to_string_lossy().to_string(),
            path,
            modified_unix,
            size_bytes: metadata.len(),
        });
    }

    records.sort_by(|left, right| {
        right
            .modified_unix
            .cmp(&left.modified_unix)
            .then_with(|| left.filename.cmp(&right.filename))
    });
    Ok(records)
}

pub fn read_planned_history_preview(
    project_root: impl AsRef<Path>,
    preview_path: impl AsRef<Path>,
) -> io::Result<PlannedHistoryPreviewDetail> {
    let preview_dir = project_root.as_ref().join("history").join("previews");
    let preview_dir = fs::canonicalize(preview_dir)?;
    let preview_path = fs::canonicalize(preview_path)?;

    if !preview_path.starts_with(&preview_dir)
        || preview_path.extension().and_then(|value| value.to_str()) != Some("toml")
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "planned history preview path is outside project preview directory",
        ));
    }

    let metadata = fs::metadata(&preview_path)?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "planned history preview path is not a file",
        ));
    }

    let modified_unix = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let record_toml = fs::read_to_string(&preview_path)?;
    let filename = preview_path
        .file_name()
        .map(|filename| filename.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(PlannedHistoryPreviewDetail {
        entry: PlannedHistoryPreviewEntry {
            path: preview_path,
            filename,
            modified_unix,
            size_bytes: metadata.len(),
        },
        record_toml,
    })
}

pub fn read_completed_history_record(
    project_root: impl AsRef<Path>,
    record_path: impl AsRef<Path>,
) -> io::Result<CompletedHistoryDetail> {
    let history_dir = project_root.as_ref().join("history").join("completed");
    let history_dir = fs::canonicalize(history_dir)?;
    let record_path = fs::canonicalize(record_path)?;

    if !record_path.starts_with(&history_dir)
        || record_path.extension().and_then(|value| value.to_str()) != Some("toml")
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "completed history record path is outside project completed history directory",
        ));
    }

    let metadata = fs::metadata(&record_path)?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "completed history record path is not a file",
        ));
    }

    let modified_unix = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let record_toml = fs::read_to_string(&record_path)?;
    let filename = record_path
        .file_name()
        .map(|filename| filename.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(CompletedHistoryDetail {
        entry: CompletedHistoryEntry {
            path: record_path,
            filename,
            modified_unix,
            size_bytes: metadata.len(),
        },
        record_toml,
    })
}

fn safe_filename_segment(value: &str) -> String {
    let mut segment = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            segment.push(character);
            previous_dash = false;
        } else if !previous_dash {
            segment.push('-');
            previous_dash = true;
        }
    }

    let segment = segment.trim_matches('-').to_string();
    if segment.is_empty() {
        "unknown".to_string()
    } else {
        segment
    }
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use waystone_publish_plan::dry_run_publish_with_context;
    use waystone_publish_plan::PublishContext;

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    #[test]
    fn creates_planned_history_record_from_dry_run() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            },
        )
        .expect("dry-run should plan");

        let record = PublicationHistoryRecord::planned_from_dry_run(&plan, "2026-07-17T00:00:00Z");

        assert_eq!(record.schema, HISTORY_SCHEMA);
        assert_eq!(record.transfer_result, "planned");
        assert_eq!(record.verification_result, "not-run");
        assert!(record
            .files
            .iter()
            .any(|file| file.path == "content/index.gmi"));
    }

    #[test]
    fn creates_completed_history_record_from_dry_run_with_reported_results() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            },
        )
        .expect("dry-run should plan");

        let record = PublicationHistoryRecord::completed_from_dry_run(
            &plan,
            "2026-07-20T00:00:00Z",
            CompletedHistoryOptions {
                transfer_result: "completed".to_string(),
                verification_result: "passed".to_string(),
                rollback_available: true,
                rollback_notes: "Remote snapshot retained".to_string(),
            },
        );

        assert_eq!(record.schema, HISTORY_SCHEMA);
        assert_eq!(record.transfer_result, "completed");
        assert_eq!(record.verification_result, "passed");
        assert!(record.rollback.available);
        assert_eq!(record.rollback.notes, "Remote snapshot retained");
        assert!(record
            .files
            .iter()
            .any(|file| file.action == "planned-upload"));
    }

    #[test]
    fn renders_inspectable_toml() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            },
        )
        .expect("dry-run should plan");

        let record = PublicationHistoryRecord::planned_from_dry_run(&plan, "2026-07-17T00:00:00Z");
        let toml = record.to_toml();

        assert!(toml.contains("[publication]"));
        assert!(toml.contains("transfer_result = \"planned\""));
        assert!(toml.contains("[[files]]"));
        assert!(toml.contains("[rollback]"));
    }

    #[test]
    fn writes_planned_history_preview_under_project_history_previews() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-preview-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("temp project root should be created");

        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            },
        )
        .expect("dry-run should plan");
        let record = PublicationHistoryRecord::planned_from_dry_run(&plan, "2026-07-19T00:00:00Z");

        let output =
            write_planned_history_preview(&root, &record).expect("preview should be written");
        assert!(output.starts_with(root.join("history").join("previews")));
        assert_eq!(
            output.file_name().and_then(|name| name.to_str()),
            Some("2026-07-19T00-00-00Z-production-planned.toml")
        );
        assert!(fs::read_to_string(output)
            .expect("preview should be readable")
            .contains("transfer_result = \"planned\""));
    }

    #[test]
    fn writes_completed_history_record_under_project_history_completed() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-completed-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("temp project root should be created");

        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            },
        )
        .expect("dry-run should plan");
        let record = PublicationHistoryRecord::completed_from_dry_run(
            &plan,
            "2026-07-20T00:00:00Z",
            CompletedHistoryOptions {
                transfer_result: "completed".to_string(),
                verification_result: "passed".to_string(),
                rollback_available: false,
                rollback_notes: "No rollback snapshot recorded".to_string(),
            },
        );

        let output =
            write_completed_history_record(&root, &record).expect("record should be written");
        assert!(output.starts_with(root.join("history").join("completed")));
        assert_eq!(
            output.file_name().and_then(|name| name.to_str()),
            Some("2026-07-20T00-00-00Z-production-completed.toml")
        );
        assert!(fs::read_to_string(output)
            .expect("record should be readable")
            .contains("transfer_result = \"completed\""));
    }

    #[test]
    fn lists_planned_history_previews_under_project_history_previews() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-preview-list-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let previews_dir = root.join("history").join("previews");
        fs::create_dir_all(&previews_dir).expect("preview directory should be created");
        fs::write(
            previews_dir.join("2026-07-19T00-00-00Z-export-planned.toml"),
            "one",
        )
        .expect("preview should be written");
        fs::write(previews_dir.join("ignore.tmp"), "two").expect("tmp should be written");

        let previews = list_planned_history_previews(&root).expect("previews should list");
        assert_eq!(previews.len(), 1);
        assert_eq!(
            previews[0].filename,
            "2026-07-19T00-00-00Z-export-planned.toml"
        );
        assert!(previews[0].path.starts_with(previews_dir));
        assert_eq!(previews[0].size_bytes, 3);
    }

    #[test]
    fn lists_completed_history_records_under_project_history_completed() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-completed-list-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let history_dir = root.join("history").join("completed");
        fs::create_dir_all(&history_dir).expect("completed history directory should be created");
        fs::write(
            history_dir.join("2026-07-20T00-00-00Z-export-completed.toml"),
            "one",
        )
        .expect("record should be written");
        fs::write(history_dir.join("ignore.tmp"), "two").expect("tmp should be written");

        let records = list_completed_history_records(&root).expect("records should list");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].filename,
            "2026-07-20T00-00-00Z-export-completed.toml"
        );
        assert!(records[0].path.starts_with(history_dir));
        assert_eq!(records[0].size_bytes, 3);
    }

    #[test]
    fn reads_planned_history_preview_detail_inside_project_boundary() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-preview-read-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let previews_dir = root.join("history").join("previews");
        fs::create_dir_all(&previews_dir).expect("preview directory should be created");
        let preview_path = previews_dir.join("2026-07-19T00-00-00Z-export-planned.toml");
        fs::write(
            &preview_path,
            "[publication]\ntransfer_result = \"planned\"\n",
        )
        .expect("preview should be written");

        let detail =
            read_planned_history_preview(&root, &preview_path).expect("preview should read");
        assert_eq!(
            detail.entry.filename,
            "2026-07-19T00-00-00Z-export-planned.toml"
        );
        assert!(detail.entry.path.starts_with(previews_dir));
        assert!(detail.record_toml.contains("transfer_result = \"planned\""));
    }

    #[test]
    fn reads_completed_history_record_detail_inside_project_boundary() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-completed-read-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let history_dir = root.join("history").join("completed");
        fs::create_dir_all(&history_dir).expect("completed history directory should be created");
        let record_path = history_dir.join("2026-07-20T00-00-00Z-export-completed.toml");
        fs::write(
            &record_path,
            "[publication]\ntransfer_result = \"completed\"\n",
        )
        .expect("record should be written");

        let detail =
            read_completed_history_record(&root, &record_path).expect("record should read");
        assert_eq!(
            detail.entry.filename,
            "2026-07-20T00-00-00Z-export-completed.toml"
        );
        assert!(detail.entry.path.starts_with(history_dir));
        assert!(detail
            .record_toml
            .contains("transfer_result = \"completed\""));
    }

    #[test]
    fn rejects_planned_history_preview_detail_outside_project_boundary() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-preview-reject-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let previews_dir = root.join("history").join("previews");
        fs::create_dir_all(&previews_dir).expect("preview directory should be created");
        let outside = root.join("outside.toml");
        fs::write(&outside, "[publication]\n").expect("outside file should be written");

        let error = read_planned_history_preview(&root, outside)
            .expect_err("outside preview should be rejected");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn rejects_completed_history_record_detail_outside_project_boundary() {
        let root = std::env::temp_dir().join(format!(
            "waystone-history-completed-reject-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        let history_dir = root.join("history").join("completed");
        fs::create_dir_all(&history_dir).expect("completed history directory should be created");
        let outside = root.join("outside.toml");
        fs::write(&outside, "[publication]\n").expect("outside file should be written");

        let error = read_completed_history_record(&root, outside)
            .expect_err("outside record should be rejected");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }
}
