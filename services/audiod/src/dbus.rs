use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use waystone_audio_service::{
    AttachRecordingRequest, AudioService, CaptureRecordingRequest, ExportOpusRequest,
    GenerateFeedRequest, InspectRecordingRequest, ListRecordingsRequest, PrepareFeedEntryRequest,
    UpdateFeedEntryRequest, UpdateRecordingRequest, ValidateFeedEntryRequest,
    ValidatePublicationRequest, ValidateRecordingRequest,
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

#[derive(Debug, Deserialize)]
struct AttachRecordingDbusRequest {
    project_root: PathBuf,
    metadata_root: String,
    id: String,
    title: String,
    master: String,
    published: String,
    feed: String,
    entry_id: String,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct UpdateRecordingDbusRequest {
    project_root: PathBuf,
    recording_metadata_path: PathBuf,
    title: String,
    master: String,
    published: String,
    feed: String,
    entry_id: String,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct CaptureRecordingDbusRequest {
    project_root: PathBuf,
    masters_root: String,
    master: String,
    duration_seconds: u32,
    input_format: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct ExportOpusDbusRequest {
    project_root: PathBuf,
    master: String,
    published: String,
    preset: String,
}

#[derive(Debug, Deserialize)]
struct FeedEntryDbusRequest {
    project_root: PathBuf,
    recording_metadata_path: PathBuf,
    updated: String,
    summary: String,
}

#[derive(Debug, Deserialize)]
struct ValidatePublicationDbusRequest {
    project_root: PathBuf,
    recording_metadata_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ValidateFeedEntryDbusRequest {
    project_root: PathBuf,
    feed_entry_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct GenerateFeedDbusRequest {
    project_root: PathBuf,
    feed_path: String,
    title: String,
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

    fn attach_recording(&self, request: &str) -> String {
        let request: AttachRecordingDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.attach_recording(AttachRecordingRequest {
            project_root: request.project_root,
            metadata_root: request.metadata_root,
            id: request.id,
            title: request.title,
            master: request.master,
            published: request.published,
            feed: request.feed,
            entry_id: request.entry_id,
            mime_type: request.mime_type,
        }) {
            Ok(attached) => success_response(json!({
                "id": attached.id,
                "title": attached.title,
                "metadata_path": attached.metadata_path,
                "metadata_relative_path": attached.metadata_relative_path,
                "master": attached.master,
                "published": attached.published,
                "feed": attached.feed,
                "entry_id": attached.entry_id,
                "mime_type": attached.mime_type,
            })),
            Err(error) => error_response("recording_attach_failed", &error.to_string()),
        }
    }

    fn update_recording(&self, request: &str) -> String {
        let request: UpdateRecordingDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.update_recording(UpdateRecordingRequest {
            project_root: request.project_root,
            recording_metadata_path: request.recording_metadata_path,
            title: request.title,
            master: request.master,
            published: request.published,
            feed: request.feed,
            entry_id: request.entry_id,
            mime_type: request.mime_type,
        }) {
            Ok(updated) => success_response(json!({
                "id": updated.id,
                "title": updated.title,
                "metadata_path": updated.metadata_path,
                "metadata_relative_path": updated.metadata_relative_path,
                "master": updated.master,
                "published": updated.published,
                "feed": updated.feed,
                "entry_id": updated.entry_id,
                "mime_type": updated.mime_type,
            })),
            Err(error) => error_response("recording_update_failed", &error.to_string()),
        }
    }

    fn capture_recording(&self, request: &str) -> String {
        let request: CaptureRecordingDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.capture_recording(CaptureRecordingRequest {
            project_root: request.project_root,
            masters_root: request.masters_root,
            master: request.master,
            duration_seconds: request.duration_seconds,
            input_format: request.input_format,
            input: request.input,
        }) {
            Ok(captured) => success_response(json!({
                "master": captured.master,
                "output_path": captured.output_path,
                "output_relative_path": captured.output_relative_path,
                "duration_seconds": captured.duration_seconds,
                "channels": captured.channels,
                "sample_rate": captured.sample_rate,
                "format": captured.format,
                "engine": captured.engine,
            })),
            Err(error) => error_response("recording_capture_failed", &error.to_string()),
        }
    }

    fn export_opus(&self, request: &str) -> String {
        let request: ExportOpusDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.export_opus(ExportOpusRequest {
            project_root: request.project_root,
            master: request.master,
            published: request.published,
            preset: request.preset,
        }) {
            Ok(exported) => success_response(json!({
                "master": exported.master,
                "published": exported.published,
                "output_path": exported.output_path,
                "output_relative_path": exported.output_relative_path,
                "preset": exported.preset,
                "mime_type": exported.mime_type,
                "engine": exported.engine,
            })),
            Err(error) => error_response("opus_export_failed", &error.to_string()),
        }
    }

    fn prepare_feed_entry(&self, request: &str) -> String {
        let request: FeedEntryDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.prepare_feed_entry(PrepareFeedEntryRequest {
            project_root: request.project_root,
            recording_metadata_path: request.recording_metadata_path,
            updated: request.updated,
            summary: request.summary,
        }) {
            Ok(prepared) => success_response(json!({
                "recording_id": prepared.recording_id,
                "title": prepared.title,
                "entry_id": prepared.entry_id,
                "feed": prepared.feed,
                "output_path": prepared.output_path,
                "output_relative_path": prepared.output_relative_path,
                "published": prepared.published,
                "mime_type": prepared.mime_type,
                "updated": prepared.updated,
            })),
            Err(error) => error_response("feed_entry_prepare_failed", &error.to_string()),
        }
    }

    fn update_feed_entry(&self, request: &str) -> String {
        let request: FeedEntryDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.update_feed_entry(UpdateFeedEntryRequest {
            project_root: request.project_root,
            recording_metadata_path: request.recording_metadata_path,
            updated: request.updated,
            summary: request.summary,
        }) {
            Ok(updated) => success_response(json!({
                "recording_id": updated.recording_id,
                "title": updated.title,
                "entry_id": updated.entry_id,
                "feed": updated.feed,
                "output_path": updated.output_path,
                "output_relative_path": updated.output_relative_path,
                "published": updated.published,
                "mime_type": updated.mime_type,
                "updated": updated.updated,
            })),
            Err(error) => error_response("feed_entry_update_failed", &error.to_string()),
        }
    }

    fn validate_publication(&self, request: &str) -> String {
        let request: ValidatePublicationDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self
            .service
            .validate_publication(ValidatePublicationRequest {
                project_root: request.project_root,
                recording_metadata_path: request.recording_metadata_path,
            }) {
            Ok(report) => validation_response(report),
            Err(error) => error_response("publication_validate_failed", &error.to_string()),
        }
    }

    fn validate_feed_entry(&self, request: &str) -> String {
        let request: ValidateFeedEntryDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.validate_feed_entry(ValidateFeedEntryRequest {
            project_root: request.project_root,
            feed_entry_path: request.feed_entry_path,
        }) {
            Ok(report) => validation_response(report),
            Err(error) => error_response("feed_entry_validate_failed", &error.to_string()),
        }
    }

    fn generate_feed(&self, request: &str) -> String {
        let request: GenerateFeedDbusRequest = match parse_request(request) {
            Ok(request) => request,
            Err(error) => return error_response("invalid_request", &error),
        };

        match self.service.generate_feed(GenerateFeedRequest {
            project_root: request.project_root,
            feed_path: request.feed_path,
            title: request.title,
        }) {
            Ok(generated) => success_response(json!({
                "feed_path": generated.feed_path,
                "feed_relative_path": generated.feed_relative_path,
                "entries": generated.entries,
                "updated": generated.updated,
            })),
            Err(error) => error_response("feed_generate_failed", &error.to_string()),
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
    parse_request(request)
}

fn parse_request<T: for<'de> Deserialize<'de>>(request: &str) -> Result<T, String> {
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

fn validation_response(report: waystone_audio_service::ValidationReport) -> String {
    success_response(json!({
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
    }))
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
