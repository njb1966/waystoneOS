use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_project_service::{
    CreateProjectRequest, InspectProjectRequest, ListProjectsRequest, ProjectService,
    ValidateProjectRequest,
};
use zbus::{blocking::connection, interface};

const BUS_NAME: &str = "org.waystone.Project1";
const OBJECT_PATH: &str = "/org/waystone/Project";

#[derive(Debug, Default)]
pub struct ProjectDbus {
    service: ProjectService,
}

#[derive(Debug, Deserialize)]
struct RootRequest {
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CreateRequest {
    parent: PathBuf,
    id: String,
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    content_index: String,
    language: Option<String>,
    author: Option<String>,
}

#[interface(name = "org.waystone.Project1")]
impl ProjectDbus {
    fn create_project(&self, request: &str) -> String {
        let request = match parse_create_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.create_project(CreateProjectRequest {
            parent: request.parent,
            id: request.id,
            name: request.name,
            project_type: request.project_type,
            content_index: request.content_index,
            language: request.language,
            author: request.author,
        }) {
            Ok(created) => success_response(json!({
                "project_path": created.project_path,
                "project_schema": created.schema,
            })),
            Err(error) => error_response("project_create_failed", &error.to_string()),
        }
    }

    fn list_projects(&self, request: &str) -> String {
        let request = match parse_root_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .list_projects(ListProjectsRequest { root: request.root })
        {
            Ok(projects) => success_response(json!({
                "projects": projects.into_iter().map(|project| {
                    json!({
                        "id": project.id,
                        "name": project.name,
                        "type": project.project_type,
                        "schema": project.schema,
                        "path": project.path,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("project_list_failed", &error.to_string()),
        }
    }

    fn inspect_project(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .inspect_project(InspectProjectRequest { path: request.path })
        {
            Ok(project) => success_response(json!({
                "id": project.id,
                "name": project.name,
                "type": project.project_type,
                "project_schema": project.schema,
                "content_root": project.content_root,
                "content_index": project.content_index,
                "publish_targets": project.publish_targets,
                "warnings": project.warnings.into_iter().map(|warning| {
                    json!({
                        "code": warning.code,
                        "message": warning.message,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("project_inspect_failed", &error.to_string()),
        }
    }

    fn validate_project(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_project(ValidateProjectRequest { path: request.path })
        {
            Ok(report) => success_response(json!({
                "valid": report.valid,
                "errors": report.errors.into_iter().map(|issue| {
                    json!({
                        "code": issue.code,
                        "message": issue.message,
                    })
                }).collect::<Vec<_>>(),
                "warnings": report.warnings.into_iter().map(|issue| {
                    json!({
                        "code": issue.code,
                        "message": issue.message,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("project_validate_failed", &error.to_string()),
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _connection = connection::Builder::session()?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, ProjectDbus::default())?
        .build()?;
    println!("waystone-projectd: serving {BUS_NAME} at {OBJECT_PATH}");

    loop {
        std::thread::park();
    }
}

fn parse_create_request(request: &str) -> Result<CreateRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_root_request(request: &str) -> Result<RootRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn parse_path_request(request: &str) -> Result<PathRequest, String> {
    serde_json::from_str(request).map_err(|error| error.to_string())
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
