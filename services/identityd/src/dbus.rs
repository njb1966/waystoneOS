use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_identity_service::{
    IdentityService, InspectIdentityRequest, ListIdentitiesRequest, ValidateIdentityRequest,
};
use zbus::{blocking::connection, interface};

const BUS_NAME: &str = "org.waystone.Identity1";
const OBJECT_PATH: &str = "/org/waystone/Identity";

#[derive(Debug, Default)]
pub struct IdentityDbus {
    service: IdentityService,
}

#[derive(Debug, Deserialize)]
struct RootRequest {
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    path: PathBuf,
}

#[interface(name = "org.waystone.Identity1")]
impl IdentityDbus {
    fn list_identities(&self, request: &str) -> String {
        let request = match parse_root_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .list_identities(ListIdentitiesRequest { root: request.root })
        {
            Ok(identities) => success_response(json!({
                "identities": identities.into_iter().map(|identity| {
                    json!({
                        "id": identity.id,
                        "display_name": identity.display_name,
                        "path": identity.path,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("identity_list_failed", &error.to_string()),
        }
    }

    fn inspect_identity(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .inspect_identity(InspectIdentityRequest { path: request.path })
        {
            Ok(identity) => success_response(json!({
                "id": identity.identity.id,
                "display_name": identity.identity.display_name,
                "author_name": identity.identity.author_name,
                "ssh_keys": identity.ssh_keys.into_iter().map(|key| {
                    json!({
                        "id": key.id,
                        "public_key": key.public_key,
                        "private_key_ref": key.private_key_ref,
                    })
                }).collect::<Vec<_>>(),
                "certificates": identity.certificates.into_iter().map(|certificate| {
                    json!({
                        "id": certificate.id,
                        "type": certificate.certificate_type,
                        "fingerprint": certificate.fingerprint,
                        "private_key_ref": certificate.private_key_ref,
                        "expires": certificate.expires,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("identity_inspect_failed", &error.to_string()),
        }
    }

    fn validate_identity(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_identity(ValidateIdentityRequest { path: request.path })
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
            Err(error) => error_response("identity_validate_failed", &error.to_string()),
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _connection = connection::Builder::session()?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, IdentityDbus::default())?
        .build()?;
    println!("waystone-identityd: serving {BUS_NAME} at {OBJECT_PATH}");

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
