use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_publish_service::{
    BuildCompletedHistoryRequest, BuildPlannedHistoryRequest, ListCompletedHistoryRequest,
    PreviewPublicationRequest, PublishService, ReadCompletedHistoryRequest,
    SaveCompletedHistoryRequest, TransferIntentRequest, ValidatePublicationRequest,
};
use zbus::{blocking::connection, interface};

const BUS_NAME: &str = "org.waystone.Publish1";
const OBJECT_PATH: &str = "/org/waystone/Publish";

#[derive(Debug, Default)]
pub struct PublishDbus {
    service: PublishService,
}

#[derive(Debug, Deserialize)]
struct PreviewRequest {
    project_path: PathBuf,
    target: String,
    hosts_root: Option<PathBuf>,
    identities_root: Option<PathBuf>,
    remote_state_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct PlannedHistoryRequest {
    project_path: PathBuf,
    target: String,
    hosts_root: Option<PathBuf>,
    identities_root: Option<PathBuf>,
    remote_state_path: Option<PathBuf>,
    date: String,
}

#[derive(Debug, Deserialize)]
struct CompletedHistoryRequest {
    project_path: PathBuf,
    target: String,
    hosts_root: Option<PathBuf>,
    identities_root: Option<PathBuf>,
    remote_state_path: Option<PathBuf>,
    date: String,
    transfer_result: String,
    verification_result: String,
    rollback_available: bool,
    rollback_notes: String,
}

#[derive(Debug, Deserialize)]
struct ProjectHistoryRequest {
    project_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CompletedHistoryReadRequest {
    project_path: PathBuf,
    record_path: PathBuf,
}

#[interface(name = "org.waystone.Publish1")]
impl PublishDbus {
    fn preview_publication(&self, request: &str) -> String {
        let request = match parse_preview_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.preview_plan(request) {
            Ok(plan) => success_response(plan_response(plan)),
            Err(error) => error_response("publication_preview_failed", &error.to_string()),
        }
    }

    fn validate_publication(&self, request: &str) -> String {
        let request = match parse_preview_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_publication(ValidatePublicationRequest {
                project_path: request.project_path,
                target: request.target,
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
                remote_state_path: request.remote_state_path,
            }) {
            Ok(validation) => success_response(validation_report_response(validation.report)),
            Err(error) => error_response("publication_validation_failed", &error.to_string()),
        }
    }

    fn transfer_intent(&self, request: &str) -> String {
        let request = match parse_preview_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.transfer_intent(TransferIntentRequest {
            project_path: request.project_path,
            target: request.target,
            hosts_root: request.hosts_root,
            identities_root: request.identities_root,
            remote_state_path: request.remote_state_path,
        }) {
            Ok(intent) => success_response(transfer_intent_response(intent.intent)),
            Err(error) => error_response("transfer_intent_failed", &error.to_string()),
        }
    }

    fn build_planned_history(&self, request: &str) -> String {
        let request = match parse_planned_history_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        let preview = match self.preview_plan(PreviewRequest {
            project_path: request.project_path,
            target: request.target,
            hosts_root: request.hosts_root,
            identities_root: request.identities_root,
            remote_state_path: request.remote_state_path,
        }) {
            Ok(plan) => plan,
            Err(error) => return error_response("publication_preview_failed", &error.to_string()),
        };

        let history = self
            .service
            .build_planned_history(BuildPlannedHistoryRequest {
                plan: preview.plan,
                date: request.date,
            });

        success_response(history_record_response(history.record))
    }

    fn build_completed_history(&self, request: &str) -> String {
        let request = match parse_completed_history_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        if !is_valid_transfer_result(&request.transfer_result) {
            return error_response(
                "invalid_request",
                "transfer_result must be completed, failed, or skipped",
            );
        }

        if !is_valid_verification_result(&request.verification_result) {
            return error_response(
                "invalid_request",
                "verification_result must be not-run, passed, or failed",
            );
        }

        let preview = match self.preview_plan(PreviewRequest {
            project_path: request.project_path,
            target: request.target,
            hosts_root: request.hosts_root,
            identities_root: request.identities_root,
            remote_state_path: request.remote_state_path,
        }) {
            Ok(plan) => plan,
            Err(error) => return error_response("publication_preview_failed", &error.to_string()),
        };

        let history = self
            .service
            .build_completed_history(BuildCompletedHistoryRequest {
                plan: preview.plan,
                date: request.date,
                transfer_result: request.transfer_result,
                verification_result: request.verification_result,
                rollback_available: request.rollback_available,
                rollback_notes: request.rollback_notes,
            });

        success_response(history_record_response(history.record))
    }

    fn save_completed_history(&self, request: &str) -> String {
        let request = match parse_completed_history_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        if !is_valid_transfer_result(&request.transfer_result) {
            return error_response(
                "invalid_request",
                "transfer_result must be completed, failed, or skipped",
            );
        }

        if !is_valid_verification_result(&request.verification_result) {
            return error_response(
                "invalid_request",
                "verification_result must be not-run, passed, or failed",
            );
        }

        let preview = match self.preview_plan(PreviewRequest {
            project_path: request.project_path.clone(),
            target: request.target.clone(),
            hosts_root: request.hosts_root,
            identities_root: request.identities_root,
            remote_state_path: request.remote_state_path,
        }) {
            Ok(plan) => plan,
            Err(error) => return error_response("publication_preview_failed", &error.to_string()),
        };

        match self
            .service
            .save_completed_history(SaveCompletedHistoryRequest {
                project_path: request.project_path,
                plan: preview.plan,
                date: request.date,
                transfer_result: request.transfer_result,
                verification_result: request.verification_result,
                rollback_available: request.rollback_available,
                rollback_notes: request.rollback_notes,
            }) {
            Ok(saved) => success_response(json!({
                "output_path": saved.output_path.display().to_string(),
                "record": history_record_value(saved.record),
            })),
            Err(error) => error_response("completed_history_save_failed", &error.to_string()),
        }
    }

    fn list_completed_history(&self, request: &str) -> String {
        let request = match parse_project_history_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .list_completed_history(ListCompletedHistoryRequest {
                project_path: request.project_path,
            }) {
            Ok(list) => success_response(json!({
                "project_path": list.project_path.display().to_string(),
                "records": list.records.into_iter().map(completed_history_entry_response).collect::<Vec<_>>(),
            })),
            Err(error) => error_response("completed_history_list_failed", &error.to_string()),
        }
    }

    fn read_completed_history(&self, request: &str) -> String {
        let request = match parse_completed_history_read_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };
        let project_path = request.project_path.clone();

        match self
            .service
            .read_completed_history(ReadCompletedHistoryRequest {
                project_path: request.project_path,
                record_path: request.record_path,
            }) {
            Ok(read) => {
                success_response(completed_history_detail_response(project_path, read.detail))
            }
            Err(error) => error_response("completed_history_read_failed", &error.to_string()),
        }
    }
}

impl PublishDbus {
    fn preview_plan(
        &self,
        request: PreviewRequest,
    ) -> Result<
        waystone_publish_service::PreviewPublicationResponse,
        waystone_publish_plan::PublishPlanError,
    > {
        self.service.preview_publication(PreviewPublicationRequest {
            project_path: request.project_path,
            target: request.target,
            hosts_root: request.hosts_root,
            identities_root: request.identities_root,
            remote_state_path: request.remote_state_path,
        })
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _connection = connection::Builder::session()?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, PublishDbus::default())?
        .build()?;
    println!("waystone-publishd: serving {BUS_NAME} at {OBJECT_PATH}");

    loop {
        std::thread::park();
    }
}

fn parse_preview_request(request: &str) -> Result<PreviewRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_planned_history_request(request: &str) -> Result<PlannedHistoryRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_completed_history_request(request: &str) -> Result<CompletedHistoryRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_project_history_request(request: &str) -> Result<ProjectHistoryRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_completed_history_read_request(
    request: &str,
) -> Result<CompletedHistoryReadRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn is_valid_transfer_result(result: &str) -> bool {
    matches!(result, "completed" | "failed" | "skipped")
}

fn is_valid_verification_result(result: &str) -> bool {
    matches!(result, "not-run" | "passed" | "failed")
}

fn plan_response(
    preview: waystone_publish_service::PreviewPublicationResponse,
) -> serde_json::Value {
    let plan = preview.plan;
    json!({
        "project": plan.project_id,
        "target": plan.target,
        "method": plan.method,
        "destination": plan.destination,
        "blocked": plan.blocked,
        "host_resolution": plan.host_resolution.map(resolution_response),
        "identity_resolution": plan.identity_resolution.map(resolution_response),
        "feed": {
            "configured": plan.feed.configured,
            "enabled": plan.feed.enabled,
            "type": plan.feed.feed_type,
            "path": plan.feed.path,
            "exists": plan.feed.exists,
            "prepared_entries": plan.feed.prepared_entries,
            "invalid_entries": plan.feed.invalid_entries,
            "invalid_entry_diagnostics": plan.feed.invalid_entry_diagnostics.into_iter().map(|diagnostic| {
                json!({
                    "path": diagnostic.path,
                    "issues": diagnostic.issues,
                })
            }).collect::<Vec<_>>(),
        },
        "comparison": {
            "configured": plan.comparison.configured,
            "source": plan.comparison.source,
            "remote_paths": plan.comparison.remote_paths,
        },
        "changes": {
            "upload": plan.upload,
            "update": plan.update,
            "delete": plan.delete,
            "skip": plan.skip,
        },
        "verification": {
            "checks": plan.verification_checks,
        },
        "confirmations": plan.confirmations,
    })
}

fn history_record_response(
    record: waystone_publication_history::PublicationHistoryRecord,
) -> serde_json::Value {
    json!({
        "record": history_record_value(record)
    })
}

fn history_record_value(
    record: waystone_publication_history::PublicationHistoryRecord,
) -> serde_json::Value {
    json!({
        "schema": record.schema,
        "date": record.date,
        "project_id": record.project_id,
        "target": record.target,
        "method": record.method,
        "destination": record.destination,
        "transfer_result": record.transfer_result,
        "verification_result": record.verification_result,
        "files": record.files.into_iter().map(|file| {
            json!({
                "path": file.path,
                "action": file.action,
                "sha256": file.sha256,
            })
        }).collect::<Vec<_>>(),
        "rollback": {
            "available": record.rollback.available,
            "notes": record.rollback.notes,
        }
    })
}

fn transfer_intent_response(intent: waystone_publish_plan::TransferIntent) -> serde_json::Value {
    json!({
        "project": intent.project_id,
        "target": intent.target,
        "method": intent.method,
        "destination": intent.destination,
        "execution_ready": intent.execution_ready,
        "blocked_reasons": intent.blocked_reasons.into_iter().map(validation_issue_response).collect::<Vec<_>>(),
        "confirmations": intent.confirmations,
        "host_resolution": intent.host_resolution.map(resolution_response),
        "identity_resolution": intent.identity_resolution.map(resolution_response),
        "comparison": {
            "configured": intent.comparison.configured,
            "source": intent.comparison.source,
            "remote_paths": intent.comparison.remote_paths,
        },
        "changes": {
            "upload": intent.upload,
            "update": intent.update,
            "delete": intent.delete,
            "skip": intent.skip,
        },
        "history": {
            "completed_directory": intent.completed_history_dir,
        },
    })
}

fn completed_history_entry_response(
    entry: waystone_publication_history::CompletedHistoryEntry,
) -> serde_json::Value {
    json!({
        "path": entry.path.display().to_string(),
        "filename": entry.filename,
        "modified_unix": entry.modified_unix,
        "size_bytes": entry.size_bytes,
    })
}

fn completed_history_detail_response(
    project_path: PathBuf,
    detail: waystone_publication_history::CompletedHistoryDetail,
) -> serde_json::Value {
    json!({
        "project_path": project_path.display().to_string(),
        "path": detail.entry.path.display().to_string(),
        "filename": detail.entry.filename,
        "modified_unix": detail.entry.modified_unix,
        "size_bytes": detail.entry.size_bytes,
        "record_toml": detail.record_toml,
    })
}

fn validation_report_response(
    report: waystone_publish_plan::PublishValidationReport,
) -> serde_json::Value {
    json!({
        "project": report.project_id,
        "target": report.target,
        "valid": report.valid,
        "blocked": report.blocked,
        "errors": report.errors.into_iter().map(validation_issue_response).collect::<Vec<_>>(),
        "warnings": report.warnings.into_iter().map(validation_issue_response).collect::<Vec<_>>(),
    })
}

fn validation_issue_response(
    issue: waystone_publish_plan::PublishValidationIssue,
) -> serde_json::Value {
    json!({
        "code": issue.code,
        "message": issue.message,
        "path": issue.path,
    })
}

fn resolution_response(resolution: waystone_publish_plan::Resolution) -> serde_json::Value {
    json!({
        "id": resolution.id,
        "status": format!("{:?}", resolution.status),
        "detail": resolution.detail,
    })
}

fn success_response(data: serde_json::Value) -> String {
    json!({
        "schema": 1,
        "ok": true,
        "data": data,
    })
    .to_string()
}

fn error_response(code: &str, message: &str) -> String {
    json!({
        "schema": 1,
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        },
    })
    .to_string()
}
