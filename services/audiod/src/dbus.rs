use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_audio_service::{
    AudioService, InspectRecordingRequest, ListRecordingsRequest, ValidateRecordingRequest,
};
use zbus::{blocking::connection, interface};

const BUS_NAME: &str = "org.waystone.Audio1";
const OBJECT_PATH: &str = "/org/waystone/Audio";

#[derive(Debug, Default)]
pub struct AudioDbus {
    service: AudioService,
}

#[derive(Debug, Deserialize)]
struct RootRequest {
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PathRequest {
    path: PathBuf,
}

#[interface(name = "org.waystone.Audio1")]
impl AudioDbus {
    fn list_recordings(&self, request: &str) -> String {
        let request = match parse_root_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .list_recordings(ListRecordingsRequest { root: request.root })
        {
            Ok(recordings) => success_response(json!({
                "recordings": recordings.into_iter().map(|recording| {
                    json!({
                        "id": recording.id,
                        "title": recording.title,
                        "path": recording.path,
                    })
                }).collect::<Vec<_>>()
            })),
            Err(error) => error_response("recording_list_failed", &error.to_string()),
        }
    }

    fn inspect_recording(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .inspect_recording(InspectRecordingRequest { path: request.path })
        {
            Ok(metadata) => success_response(json!({
                "id": metadata.recording.id,
                "title": metadata.recording.title,
                "master": metadata.recording.master,
                "published": metadata.recording.published,
                "duration_seconds": metadata.recording.duration_seconds,
                "channels": metadata.recording.channels,
                "sample_rate": metadata.recording.sample_rate,
                "publication": metadata.publication.map(|publication| {
                    json!({
                        "feed": publication.feed,
                        "entry_id": publication.entry_id,
                        "mime_type": publication.mime_type,
                    })
                }),
            })),
            Err(error) => error_response("recording_inspect_failed", &error.to_string()),
        }
    }

    fn validate_recording(&self, request: &str) -> String {
        let request = match parse_path_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_recording(ValidateRecordingRequest { path: request.path })
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
            Err(error) => error_response("recording_validate_failed", &error.to_string()),
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _connection = connection::Builder::session()?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, AudioDbus::default())?
        .build()?;
    println!("waystone-audiod: serving {BUS_NAME} at {OBJECT_PATH}");

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
