use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_host_service::{
    HostService, InspectHostRequest, ListHostsRequest, ValidateHostRequest,
};
use zbus::{blocking::connection, interface};

const BUS_NAME: &str = "org.waystone.Host1";
const OBJECT_PATH: &str = "/org/waystone/Host";

#[derive(Debug, Default)]
pub struct HostDbus {
    service: HostService,
}

#[derive(Debug, Deserialize)]
struct RootRequest {
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    path: PathBuf,
}

#[interface(name = "org.waystone.Host1")]
impl HostDbus {
    fn list_hosts(&self, request: &str) -> String {
        let request = match parse_root_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .list_hosts(ListHostsRequest { root: request.root })
        {
            Ok(hosts) => success_response(json!({
                "hosts": hosts.into_iter().map(|host| {
                    json!({
                        "id": host.id,
                        "display_name": host.display_name,
                        "address": host.address,
                        "path": host.path,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("host_list_failed", &error.to_string()),
        }
    }

    fn inspect_host(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .inspect_host(InspectHostRequest { path: request.path })
        {
            Ok(host) => success_response(json!({
                "id": host.host.id,
                "display_name": host.host.display_name,
                "address": host.host.address,
                "notes": host.host.notes,
                "services": host.services.into_iter().map(|service| {
                    json!({
                        "type": service.service_type,
                        "port": service.port,
                        "trust": service.trust,
                        "fingerprint": service.fingerprint,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("host_inspect_failed", &error.to_string()),
        }
    }

    fn validate_host(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_host(ValidateHostRequest { path: request.path })
        {
            Ok(report) => success_response(json!({
                "valid": report.valid,
                "errors": report.errors.into_iter().map(|issue| {
                    json!({
                        "code": issue.code,
                        "message": issue.message,
                        "path": issue.path,
                    })
                }).collect::<Vec<_>>(),
                "warnings": report.warnings.into_iter().map(|issue| {
                    json!({
                        "code": issue.code,
                        "message": issue.message,
                        "path": issue.path,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("host_validate_failed", &error.to_string()),
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _connection = connection::Builder::session()?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, HostDbus::default())?
        .build()?;
    println!("waystone-hostd: serving {BUS_NAME} at {OBJECT_PATH}");

    loop {
        std::thread::park();
    }
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
