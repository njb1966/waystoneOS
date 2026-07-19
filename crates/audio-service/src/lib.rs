use std::path::PathBuf;
use waystone_audio_metadata::{
    attach_recording, list_recordings, load_audio_metadata, validate_audio_metadata,
    AttachRecordingOptions, AttachedRecording, AudioMetadata, AudioMetadataError, RecordingSummary,
    ValidationReport,
};

#[derive(Debug, Default)]
pub struct AudioService;

#[derive(Debug, Clone)]
pub struct ListRecordingsRequest {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InspectRecordingRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateRecordingRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AttachRecordingRequest {
    pub project_root: PathBuf,
    pub metadata_root: String,
    pub id: String,
    pub title: String,
    pub master: String,
    pub published: String,
    pub feed: String,
    pub entry_id: String,
    pub mime_type: String,
}

impl AudioService {
    pub fn list_recordings(
        &self,
        request: ListRecordingsRequest,
    ) -> Result<Vec<RecordingSummary>, AudioMetadataError> {
        list_recordings(request.root)
    }

    pub fn inspect_recording(
        &self,
        request: InspectRecordingRequest,
    ) -> Result<AudioMetadata, AudioMetadataError> {
        load_audio_metadata(request.path)
    }

    pub fn validate_recording(
        &self,
        request: ValidateRecordingRequest,
    ) -> Result<ValidationReport, AudioMetadataError> {
        validate_audio_metadata(request.path)
    }

    pub fn attach_recording(
        &self,
        request: AttachRecordingRequest,
    ) -> Result<AttachedRecording, AudioMetadataError> {
        attach_recording(&AttachRecordingOptions {
            project_root: request.project_root,
            metadata_root: request.metadata_root,
            id: request.id,
            title: request.title,
            master: request.master,
            published: request.published,
            feed: request.feed,
            entry_id: request.entry_id,
            mime_type: request.mime_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    #[test]
    fn service_lists_and_validates_recordings() {
        let service = AudioService;
        let recordings = service
            .list_recordings(ListRecordingsRequest {
                root: repo_path("examples/projects/audio-capsule.wayproject/audio/metadata"),
            })
            .expect("recordings should list");

        assert_eq!(recordings.len(), 1);

        let report = service
            .validate_recording(ValidateRecordingRequest {
                path: recordings[0].path.clone(),
            })
            .expect("recording should validate");

        assert!(report.valid, "{report:#?}");
    }

    #[test]
    fn service_attaches_recording_metadata() {
        let root =
            std::env::temp_dir().join(format!("waystone-audio-service-{}", std::process::id()));
        let project = root.join("audio-project.wayproject");
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::create_dir_all(project.join("feeds")).expect("feeds directory");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published file");

        let service = AudioService;
        let attached = service
            .attach_recording(AttachRecordingRequest {
                project_root: project.clone(),
                metadata_root: "audio/metadata".to_string(),
                id: "note".to_string(),
                title: "Note".to_string(),
                master: "audio/masters/note.flac".to_string(),
                published: "audio/published/note.opus".to_string(),
                feed: "feeds/feed.xml".to_string(),
                entry_id: "tag:example.invalid,2026:note".to_string(),
                mime_type: "audio/ogg; codecs=opus".to_string(),
            })
            .expect("recording should attach");

        assert_eq!(attached.metadata_relative_path, "audio/metadata/note.toml");
        assert!(attached.metadata_path.is_file());

        let report = service
            .validate_recording(ValidateRecordingRequest {
                path: attached.metadata_path,
            })
            .expect("attached recording should validate");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }
}
