use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_publish_service::{
    BuildCompletedHistoryRequest, BuildPlannedHistoryRequest, PreviewPublicationRequest,
    PublishService, ValidatePublicationRequest,
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
}

#[derive(Debug, Deserialize)]
struct PlannedHistoryRequest {
    project_path: PathBuf,
    target: String,
    hosts_root: Option<PathBuf>,
    identities_root: Option<PathBuf>,
    date: String,
}

#[derive(Debug, Deserialize)]
struct CompletedHistoryRequest {
    project_path: PathBuf,
    target: String,
    hosts_root: Option<PathBuf>,
    identities_root: Option<PathBuf>,
    date: String,
    transfer_result: String,
    verification_result: String,
    rollback_available: bool,
    rollback_notes: String,
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
            }) {
            Ok(validation) => success_response(validation_report_response(validation.report)),
            Err(error) => error_response("publication_validation_failed", &error.to_string()),
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
        "record": {
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
        }
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
