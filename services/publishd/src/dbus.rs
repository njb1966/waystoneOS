use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_publish_service::{
    BuildPlannedHistoryRequest, PreviewPublicationRequest, PublishService,
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

        success_response(json!({
            "record": {
                "schema": history.record.schema,
                "date": history.record.date,
                "project_id": history.record.project_id,
                "target": history.record.target,
                "method": history.record.method,
                "destination": history.record.destination,
                "transfer_result": history.record.transfer_result,
                "verification_result": history.record.verification_result,
                "files": history.record.files.into_iter().map(|file| {
                    json!({
                        "path": file.path,
                        "action": file.action,
                        "sha256": file.sha256,
                    })
                }).collect::<Vec<_>>(),
                "rollback": {
                    "available": history.record.rollback.available,
                    "notes": history.record.rollback.notes,
                }
            }
        }))
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
