use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use waystone_publication_history::{
    list_completed_history_records, read_completed_history_record, write_completed_history_record,
    CompletedHistoryDetail, CompletedHistoryEntry, CompletedHistoryOptions, PublicationFile,
    PublicationHistoryRecord, RollbackInfo, HISTORY_SCHEMA,
};
use waystone_publish_plan::{
    dry_run_publish_with_context, prepare_removable_execution_with_context,
    transfer_intent_with_context, validate_publication_with_context, PublishContext, PublishDryRun,
    PublishPlanError, PublishValidationIssue, PublishValidationReport, RemovableExecutionOperation,
    RemovableExecutionPlan, TransferIntent,
};

#[derive(Debug, Default)]
pub struct PublishService;

#[derive(Debug, Clone)]
pub struct PreviewPublicationRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
    pub remote_state_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PreviewPublicationResponse {
    pub plan: PublishDryRun,
}

#[derive(Debug, Clone)]
pub struct ValidatePublicationRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
    pub remote_state_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ValidatePublicationResponse {
    pub report: PublishValidationReport,
}

#[derive(Debug, Clone)]
pub struct TransferIntentRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
    pub remote_state_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TransferIntentResponse {
    pub intent: TransferIntent,
}

#[derive(Debug, Clone)]
pub struct PrepareRemovableExecutionRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
    pub remote_state_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PrepareRemovableExecutionResponse {
    pub plan: RemovableExecutionPlan,
}

#[derive(Debug, Clone)]
pub struct ExecuteRemovableRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub remote_state_path: Option<PathBuf>,
    pub date: String,
    pub confirm_transfer: bool,
}

#[derive(Debug, Clone)]
pub struct ExecuteRemovableResponse {
    pub project_path: PathBuf,
    pub plan: RemovableExecutionPlan,
    pub result: RemovableExecutionResult,
    pub record: PublicationHistoryRecord,
    pub history_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RemovableExecutionResult {
    pub transfer_result: String,
    pub verification_result: String,
    pub files: Vec<RemovableExecutionFileResult>,
}

#[derive(Debug, Clone)]
pub struct RemovableExecutionFileResult {
    pub project_path: String,
    pub source_path: Option<String>,
    pub destination_path: String,
    pub action: String,
    pub result: String,
    pub bytes: Option<u64>,
}

#[derive(Debug)]
pub enum ExecuteRemovableError {
    Plan(PublishPlanError),
    ConfirmationRequired,
    PlanBlocked(Vec<PublishValidationIssue>),
    DestinationExists(PathBuf),
    UnsafeDestination {
        root: PathBuf,
        path: PathBuf,
    },
    MissingSource(PathBuf),
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    Copy {
        source_path: PathBuf,
        destination_path: PathBuf,
        source: std::io::Error,
    },
    Rename {
        temporary_path: PathBuf,
        destination_path: PathBuf,
        source: std::io::Error,
    },
    WriteHistory(std::io::Error),
}

impl fmt::Display for ExecuteRemovableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plan(error) => write!(formatter, "{error}"),
            Self::ConfirmationRequired => {
                write!(formatter, "removable execution requires --confirm-transfer")
            }
            Self::PlanBlocked(reasons) => {
                let details = reasons
                    .iter()
                    .map(|reason| format!("{}: {}", reason.code, reason.message))
                    .collect::<Vec<_>>()
                    .join("; ");
                write!(formatter, "removable execution plan is blocked: {details}")
            }
            Self::DestinationExists(path) => {
                write!(formatter, "destination already exists: {}", path.display())
            }
            Self::UnsafeDestination { root, path } => write!(
                formatter,
                "destination escaped removable root: {} is not under {}",
                path.display(),
                root.display()
            ),
            Self::MissingSource(path) => {
                write!(formatter, "source file is missing: {}", path.display())
            }
            Self::CreateDirectory { path, source } => {
                write!(
                    formatter,
                    "destination directory could not be created: {}: {source}",
                    path.display()
                )
            }
            Self::Copy {
                source_path,
                destination_path,
                source,
            } => write!(
                formatter,
                "file could not be copied: {} -> {}: {source}",
                source_path.display(),
                destination_path.display()
            ),
            Self::Rename {
                temporary_path,
                destination_path,
                source,
            } => write!(
                formatter,
                "temporary file could not be moved into place: {} -> {}: {source}",
                temporary_path.display(),
                destination_path.display()
            ),
            Self::WriteHistory(error) => {
                write!(formatter, "completed history could not be written: {error}")
            }
        }
    }
}

impl std::error::Error for ExecuteRemovableError {}

#[derive(Debug, Clone)]
pub struct BuildPlannedHistoryRequest {
    pub plan: PublishDryRun,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct BuildPlannedHistoryResponse {
    pub record: PublicationHistoryRecord,
}

#[derive(Debug, Clone)]
pub struct BuildCompletedHistoryRequest {
    pub plan: PublishDryRun,
    pub date: String,
    pub transfer_result: String,
    pub verification_result: String,
    pub rollback_available: bool,
    pub rollback_notes: String,
}

#[derive(Debug, Clone)]
pub struct BuildCompletedHistoryResponse {
    pub record: PublicationHistoryRecord,
}

#[derive(Debug, Clone)]
pub struct SaveCompletedHistoryRequest {
    pub project_path: PathBuf,
    pub plan: PublishDryRun,
    pub date: String,
    pub transfer_result: String,
    pub verification_result: String,
    pub rollback_available: bool,
    pub rollback_notes: String,
}

#[derive(Debug, Clone)]
pub struct SaveCompletedHistoryResponse {
    pub record: PublicationHistoryRecord,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ListCompletedHistoryRequest {
    pub project_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ListCompletedHistoryResponse {
    pub project_path: PathBuf,
    pub records: Vec<CompletedHistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct ReadCompletedHistoryRequest {
    pub project_path: PathBuf,
    pub record_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ReadCompletedHistoryResponse {
    pub detail: CompletedHistoryDetail,
}

impl PublishService {
    pub fn preview_publication(
        &self,
        request: PreviewPublicationRequest,
    ) -> Result<PreviewPublicationResponse, PublishPlanError> {
        let plan = dry_run_publish_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
                remote_state_path: request.remote_state_path,
            },
        )?;

        Ok(PreviewPublicationResponse { plan })
    }

    pub fn validate_publication(
        &self,
        request: ValidatePublicationRequest,
    ) -> Result<ValidatePublicationResponse, PublishPlanError> {
        let report = validate_publication_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
                remote_state_path: request.remote_state_path,
            },
        )?;

        Ok(ValidatePublicationResponse { report })
    }

    pub fn transfer_intent(
        &self,
        request: TransferIntentRequest,
    ) -> Result<TransferIntentResponse, PublishPlanError> {
        let intent = transfer_intent_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
                remote_state_path: request.remote_state_path,
            },
        )?;

        Ok(TransferIntentResponse { intent })
    }

    pub fn prepare_removable_execution(
        &self,
        request: PrepareRemovableExecutionRequest,
    ) -> Result<PrepareRemovableExecutionResponse, PublishPlanError> {
        let plan = prepare_removable_execution_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
                remote_state_path: request.remote_state_path,
            },
        )?;

        Ok(PrepareRemovableExecutionResponse { plan })
    }

    pub fn execute_removable(
        &self,
        request: ExecuteRemovableRequest,
    ) -> Result<ExecuteRemovableResponse, ExecuteRemovableError> {
        if !request.confirm_transfer {
            return Err(ExecuteRemovableError::ConfirmationRequired);
        }

        let project_path = request.project_path;
        let plan = prepare_removable_execution_with_context(
            &project_path,
            &request.target,
            &PublishContext {
                hosts_root: None,
                identities_root: None,
                remote_state_path: request.remote_state_path,
            },
        )
        .map_err(ExecuteRemovableError::Plan)?;

        if !plan.execution_ready || !plan.blocked_reasons.is_empty() {
            return Err(ExecuteRemovableError::PlanBlocked(
                plan.blocked_reasons.clone(),
            ));
        }

        preflight_removable_plan(&plan)?;
        let files = copy_removable_files(&plan)?;
        let result = RemovableExecutionResult {
            transfer_result: "completed".to_string(),
            verification_result: "not-run".to_string(),
            files,
        };
        let record = completed_history_from_removable_result(&plan, &result, request.date);
        let history_path = write_completed_history_record(&project_path, &record)
            .map_err(ExecuteRemovableError::WriteHistory)?;

        Ok(ExecuteRemovableResponse {
            project_path,
            plan,
            result,
            record,
            history_path,
        })
    }

    pub fn build_planned_history(
        &self,
        request: BuildPlannedHistoryRequest,
    ) -> BuildPlannedHistoryResponse {
        BuildPlannedHistoryResponse {
            record: PublicationHistoryRecord::planned_from_dry_run(&request.plan, request.date),
        }
    }

    pub fn build_completed_history(
        &self,
        request: BuildCompletedHistoryRequest,
    ) -> BuildCompletedHistoryResponse {
        BuildCompletedHistoryResponse {
            record: PublicationHistoryRecord::completed_from_dry_run(
                &request.plan,
                request.date,
                CompletedHistoryOptions {
                    transfer_result: request.transfer_result,
                    verification_result: request.verification_result,
                    rollback_available: request.rollback_available,
                    rollback_notes: request.rollback_notes,
                },
            ),
        }
    }

    pub fn save_completed_history(
        &self,
        request: SaveCompletedHistoryRequest,
    ) -> std::io::Result<SaveCompletedHistoryResponse> {
        let record = PublicationHistoryRecord::completed_from_dry_run(
            &request.plan,
            request.date,
            CompletedHistoryOptions {
                transfer_result: request.transfer_result,
                verification_result: request.verification_result,
                rollback_available: request.rollback_available,
                rollback_notes: request.rollback_notes,
            },
        );
        let output_path = write_completed_history_record(&request.project_path, &record)?;
        Ok(SaveCompletedHistoryResponse {
            record,
            output_path,
        })
    }

    pub fn list_completed_history(
        &self,
        request: ListCompletedHistoryRequest,
    ) -> std::io::Result<ListCompletedHistoryResponse> {
        let records = list_completed_history_records(&request.project_path)?;
        Ok(ListCompletedHistoryResponse {
            project_path: request.project_path,
            records,
        })
    }

    pub fn read_completed_history(
        &self,
        request: ReadCompletedHistoryRequest,
    ) -> std::io::Result<ReadCompletedHistoryResponse> {
        let detail = read_completed_history_record(&request.project_path, &request.record_path)?;
        Ok(ReadCompletedHistoryResponse { detail })
    }
}

fn preflight_removable_plan(plan: &RemovableExecutionPlan) -> Result<(), ExecuteRemovableError> {
    let root = Path::new(&plan.destination_root);
    for operation in plan.upload.iter().chain(plan.update.iter()) {
        let Some(source_path) = &operation.source_path else {
            return Err(ExecuteRemovableError::MissingSource(PathBuf::from(
                &operation.project_path,
            )));
        };
        let source_path = PathBuf::from(source_path);
        if !source_path.is_file() {
            return Err(ExecuteRemovableError::MissingSource(source_path));
        }

        let destination_path = PathBuf::from(&operation.destination_path);
        ensure_destination_under_root(root, &destination_path)?;
        if operation_is_upload(plan, operation) && destination_path.exists() {
            return Err(ExecuteRemovableError::DestinationExists(destination_path));
        }
        let temporary_path = temporary_copy_path(&destination_path);
        if temporary_path.exists() {
            return Err(ExecuteRemovableError::DestinationExists(temporary_path));
        }
    }

    for operation in plan.delete.iter().chain(plan.skip.iter()) {
        let destination_path = PathBuf::from(&operation.destination_path);
        ensure_destination_under_root(root, &destination_path)?;
    }

    Ok(())
}

fn copy_removable_files(
    plan: &RemovableExecutionPlan,
) -> Result<Vec<RemovableExecutionFileResult>, ExecuteRemovableError> {
    let mut results = Vec::new();
    copy_operations(&plan.upload, "upload", "copied", &mut results)?;
    copy_operations(&plan.update, "update", "copied", &mut results)?;
    for operation in &plan.skip {
        results.push(file_result(operation, "skip", "skipped", None));
    }
    Ok(results)
}

fn copy_operations(
    operations: &[RemovableExecutionOperation],
    action: &str,
    result: &str,
    results: &mut Vec<RemovableExecutionFileResult>,
) -> Result<(), ExecuteRemovableError> {
    for operation in operations {
        let source_path = PathBuf::from(
            operation
                .source_path
                .as_ref()
                .expect("preflight requires copy operations to have sources"),
        );
        let destination_path = PathBuf::from(&operation.destination_path);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                ExecuteRemovableError::CreateDirectory {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        let temporary_path = temporary_copy_path(&destination_path);
        if temporary_path.exists() {
            return Err(ExecuteRemovableError::DestinationExists(temporary_path));
        }
        let bytes = fs::copy(&source_path, &temporary_path).map_err(|source| {
            let _ = fs::remove_file(&temporary_path);
            ExecuteRemovableError::Copy {
                source_path: source_path.clone(),
                destination_path: destination_path.clone(),
                source,
            }
        })?;
        fs::rename(&temporary_path, &destination_path).map_err(|source| {
            let _ = fs::remove_file(&temporary_path);
            ExecuteRemovableError::Rename {
                temporary_path: temporary_path.clone(),
                destination_path: destination_path.clone(),
                source,
            }
        })?;
        results.push(file_result(operation, action, result, Some(bytes)));
    }

    Ok(())
}

fn temporary_copy_path(destination_path: &Path) -> PathBuf {
    let file_name = destination_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("copy");
    destination_path.with_file_name(format!(".{file_name}.waystone-copy-{}.tmp", process::id()))
}

fn operation_is_upload(
    plan: &RemovableExecutionPlan,
    operation: &RemovableExecutionOperation,
) -> bool {
    plan.upload
        .iter()
        .any(|upload| upload.project_path == operation.project_path)
}

fn ensure_destination_under_root(root: &Path, path: &Path) -> Result<(), ExecuteRemovableError> {
    if path.starts_with(root) {
        Ok(())
    } else {
        Err(ExecuteRemovableError::UnsafeDestination {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    }
}

fn file_result(
    operation: &RemovableExecutionOperation,
    action: &str,
    result: &str,
    bytes: Option<u64>,
) -> RemovableExecutionFileResult {
    RemovableExecutionFileResult {
        project_path: operation.project_path.clone(),
        source_path: operation.source_path.clone(),
        destination_path: operation.destination_path.clone(),
        action: action.to_string(),
        result: result.to_string(),
        bytes,
    }
}

fn completed_history_from_removable_result(
    plan: &RemovableExecutionPlan,
    result: &RemovableExecutionResult,
    date: String,
) -> PublicationHistoryRecord {
    PublicationHistoryRecord {
        schema: HISTORY_SCHEMA,
        date,
        project_id: plan.project_id.clone(),
        target: plan.target.clone(),
        method: plan.method.clone(),
        destination: Some(plan.destination_root.clone()),
        transfer_result: result.transfer_result.clone(),
        verification_result: result.verification_result.clone(),
        files: result
            .files
            .iter()
            .map(|file| PublicationFile {
                path: file.project_path.clone(),
                action: format!("{}-{}", file.result, file.action),
                sha256: None,
            })
            .collect(),
        rollback: RollbackInfo {
            available: false,
            notes: "Removable transfer completed; no rollback snapshot recorded".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    fn temp_project_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "waystone-publish-service-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ))
    }

    fn copy_directory(from: &Path, to: &Path) {
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
    fn service_previews_and_builds_planned_history() {
        let service = PublishService;
        let preview = service
            .preview_publication(PreviewPublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            })
            .expect("preview should succeed");

        assert!(!preview.plan.blocked);

        let history = service.build_planned_history(BuildPlannedHistoryRequest {
            plan: preview.plan,
            date: "2026-07-17T00:00:00Z".to_string(),
        });

        assert_eq!(history.record.transfer_result, "planned");
    }

    #[test]
    fn service_builds_completed_history_with_reported_results() {
        let service = PublishService;
        let preview = service
            .preview_publication(PreviewPublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            })
            .expect("preview should succeed");

        let history = service.build_completed_history(BuildCompletedHistoryRequest {
            plan: preview.plan,
            date: "2026-07-20T00:00:00Z".to_string(),
            transfer_result: "completed".to_string(),
            verification_result: "passed".to_string(),
            rollback_available: false,
            rollback_notes: "No rollback snapshot recorded".to_string(),
        });

        assert_eq!(history.record.transfer_result, "completed");
        assert_eq!(history.record.verification_result, "passed");
        assert!(!history.record.rollback.available);
    }

    #[test]
    fn service_saves_lists_and_reads_completed_history() {
        let root = std::env::temp_dir().join(format!(
            "waystone-publish-service-completed-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("temp project root should be created");

        let service = PublishService;
        let preview = service
            .preview_publication(PreviewPublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            })
            .expect("preview should succeed");

        let saved = service
            .save_completed_history(SaveCompletedHistoryRequest {
                project_path: root.clone(),
                plan: preview.plan,
                date: "2026-07-20T00:00:00Z".to_string(),
                transfer_result: "completed".to_string(),
                verification_result: "passed".to_string(),
                rollback_available: false,
                rollback_notes: "No rollback snapshot recorded".to_string(),
            })
            .expect("completed history should save");

        assert!(saved
            .output_path
            .starts_with(root.join("history").join("completed")));

        let listed = service
            .list_completed_history(ListCompletedHistoryRequest {
                project_path: root.clone(),
            })
            .expect("completed history should list");
        assert_eq!(listed.records.len(), 1);
        assert_eq!(listed.records[0].path, saved.output_path);

        let read = service
            .read_completed_history(ReadCompletedHistoryRequest {
                project_path: root,
                record_path: saved.output_path,
            })
            .expect("completed history should read");
        assert!(read
            .detail
            .record_toml
            .contains("transfer_result = \"completed\""));
    }

    #[test]
    fn service_validates_publication_readiness() {
        let service = PublishService;
        let validation = service
            .validate_publication(ValidatePublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
                remote_state_path: None,
            })
            .expect("validation should succeed");

        assert!(validation.report.valid);
        assert!(!validation.report.blocked);
        assert!(validation.report.errors.is_empty());
    }

    #[test]
    fn service_builds_transfer_intent() {
        let service = PublishService;
        let intent = service
            .transfer_intent(TransferIntentRequest {
                project_path: repo_path("examples/projects/audio-capsule.wayproject"),
                target: "export".to_string(),
                hosts_root: None,
                identities_root: None,
                remote_state_path: None,
            })
            .expect("transfer intent should build");

        assert!(intent.intent.execution_ready);
        assert_eq!(intent.intent.method, "removable");
        assert!(intent
            .intent
            .completed_history_dir
            .ends_with("history/completed"));
    }

    #[test]
    fn service_prepares_removable_execution_plan() {
        let service = PublishService;
        let plan = service
            .prepare_removable_execution(PrepareRemovableExecutionRequest {
                project_path: repo_path("examples/projects/audio-capsule.wayproject"),
                target: "export".to_string(),
                hosts_root: None,
                identities_root: None,
                remote_state_path: None,
            })
            .expect("removable execution plan should build");

        assert!(plan.plan.execution_ready);
        assert_eq!(plan.plan.method, "removable");
        assert!(plan
            .plan
            .upload
            .iter()
            .any(|operation| operation.project_path == "content/index.gmi"));
    }

    #[test]
    fn service_executes_removable_copy_and_writes_completed_history() {
        let project = temp_project_root("execute-removable").join("audio-capsule.wayproject");
        copy_directory(
            &repo_path("examples/projects/audio-capsule.wayproject"),
            &project,
        );

        let service = PublishService;
        let executed = service
            .execute_removable(ExecuteRemovableRequest {
                project_path: project.clone(),
                target: "export".to_string(),
                remote_state_path: None,
                date: "2026-07-21T00:00:00Z".to_string(),
                confirm_transfer: true,
            })
            .expect("removable execution should copy files");

        assert_eq!(executed.result.transfer_result, "completed");
        assert_eq!(executed.result.verification_result, "not-run");
        assert!(project.join("publish/export/content/index.gmi").is_file());
        assert!(!temporary_copy_path(&project.join("publish/export/content/index.gmi")).exists());
        assert!(project
            .join("publish/export/audio/published/field-note.opus")
            .is_file());
        assert!(executed
            .history_path
            .starts_with(project.join("history").join("completed")));
        assert!(executed
            .record
            .files
            .iter()
            .any(|file| file.path == "content/index.gmi" && file.action == "copied-upload"));
        assert!(std::fs::read_to_string(&executed.history_path)
            .expect("completed history should be readable")
            .contains("copied-upload"));

        let _ = std::fs::remove_dir_all(project.parent().expect("temp project has parent"));
    }

    #[test]
    fn service_execute_removable_refuses_temporary_copy_collision() {
        let project =
            temp_project_root("execute-removable-temp-collision").join("audio-capsule.wayproject");
        copy_directory(
            &repo_path("examples/projects/audio-capsule.wayproject"),
            &project,
        );
        let destination = project.join("publish/export/content/index.gmi");
        let temporary = temporary_copy_path(&destination);
        std::fs::create_dir_all(temporary.parent().expect("temporary path has parent"))
            .expect("temporary destination parent");
        std::fs::write(&temporary, "stale temp").expect("temporary copy placeholder");

        let service = PublishService;
        let error = service
            .execute_removable(ExecuteRemovableRequest {
                project_path: project.clone(),
                target: "export".to_string(),
                remote_state_path: None,
                date: "2026-07-21T00:00:00Z".to_string(),
                confirm_transfer: true,
            })
            .expect_err("execution should refuse temp file collision");

        assert!(matches!(error, ExecuteRemovableError::DestinationExists(_)));
        assert!(!destination.exists());
        assert_eq!(
            std::fs::read_to_string(&temporary).expect("temporary file should remain untouched"),
            "stale temp"
        );

        let _ = std::fs::remove_dir_all(project.parent().expect("temp project has parent"));
    }

    #[test]
    fn service_execute_removable_requires_confirmation() {
        let service = PublishService;
        let error = service
            .execute_removable(ExecuteRemovableRequest {
                project_path: repo_path("examples/projects/audio-capsule.wayproject"),
                target: "export".to_string(),
                remote_state_path: None,
                date: "2026-07-21T00:00:00Z".to_string(),
                confirm_transfer: false,
            })
            .expect_err("execution should require explicit confirmation");

        assert!(matches!(error, ExecuteRemovableError::ConfirmationRequired));
    }

    #[test]
    fn service_execute_removable_refuses_upload_overwrite() {
        let project =
            temp_project_root("execute-removable-overwrite").join("audio-capsule.wayproject");
        copy_directory(
            &repo_path("examples/projects/audio-capsule.wayproject"),
            &project,
        );
        let existing = project.join("publish/export/content/index.gmi");
        std::fs::create_dir_all(existing.parent().expect("existing path has parent"))
            .expect("existing destination parent");
        std::fs::write(&existing, "existing").expect("existing destination");

        let service = PublishService;
        let error = service
            .execute_removable(ExecuteRemovableRequest {
                project_path: project.clone(),
                target: "export".to_string(),
                remote_state_path: None,
                date: "2026-07-21T00:00:00Z".to_string(),
                confirm_transfer: true,
            })
            .expect_err("execution should refuse upload overwrite");

        assert!(matches!(error, ExecuteRemovableError::DestinationExists(_)));
        assert_eq!(
            std::fs::read_to_string(&existing).expect("existing destination should remain"),
            "existing"
        );

        let _ = std::fs::remove_dir_all(project.parent().expect("temp project has parent"));
    }
}
