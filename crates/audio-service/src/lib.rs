use std::path::PathBuf;
use waystone_audio_metadata::{
    list_recordings, load_audio_metadata, validate_audio_metadata, AudioMetadata,
    AudioMetadataError, RecordingSummary, ValidationReport,
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
