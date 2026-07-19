use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct AudioMetadata {
    pub recording: RecordingSection,
    pub publication: Option<PublicationSection>,
}

#[derive(Debug, Deserialize)]
pub struct RecordingSection {
    pub id: String,
    pub title: String,
    pub master: String,
    pub published: Option<String>,
    pub duration_seconds: Option<u64>,
    pub channels: Option<u8>,
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PublicationSection {
    pub feed: Option<String>,
    pub entry_id: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn from_issues(issues: Vec<ValidationIssue>) -> Self {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for issue in issues {
            match issue.severity {
                Severity::Error => errors.push(issue),
                Severity::Warning => warnings.push(issue),
            }
        }

        Self {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordingSummary {
    pub id: String,
    pub title: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AttachRecordingOptions {
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

#[derive(Debug, Clone)]
pub struct AttachedRecording {
    pub id: String,
    pub title: String,
    pub metadata_path: PathBuf,
    pub metadata_relative_path: String,
    pub master: String,
    pub published: String,
    pub feed: String,
    pub entry_id: String,
    pub mime_type: String,
}

#[derive(Debug, Error)]
pub enum AudioMetadataError {
    #[error("project path does not exist: {0}")]
    ProjectNotFound(PathBuf),

    #[error("project path is not a directory: {0}")]
    ProjectNotDirectory(PathBuf),

    #[error("metadata path does not exist: {0}")]
    NotFound(PathBuf),

    #[error("metadata path is not a file: {0}")]
    NotFile(PathBuf),

    #[error("metadata could not be read: {path}: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("metadata could not be parsed: {0}")]
    ParseFailed(toml::de::Error),

    #[error("metadata directory does not exist: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("metadata path is not a directory: {0}")]
    NotDirectory(PathBuf),

    #[error("metadata directory could not be read: {path}: {source}")]
    ReadDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid recording id: {0}")]
    InvalidRecordingId(String),

    #[error("invalid audio path in {field}: {value}")]
    InvalidAudioPath { field: String, value: String },

    #[error("recording metadata already exists: {0}")]
    MetadataAlreadyExists(PathBuf),

    #[error("audio file is missing: {path}")]
    AudioFileMissing { path: String },

    #[error("metadata directory could not be created: {path}: {source}")]
    CreateDirectoryFailed { path: PathBuf, source: io::Error },

    #[error("metadata file could not be written: {path}: {source}")]
    WriteFileFailed { path: PathBuf, source: io::Error },

    #[error("metadata file could not be installed: {path}: {source}")]
    InstallMetadataFailed { path: PathBuf, source: io::Error },
}

pub fn load_audio_metadata(path: impl AsRef<Path>) -> Result<AudioMetadata, AudioMetadataError> {
    let text = read_metadata(path.as_ref())?;
    toml::from_str(&text).map_err(AudioMetadataError::ParseFailed)
}

pub fn validate_audio_metadata(
    path: impl AsRef<Path>,
) -> Result<ValidationReport, AudioMetadataError> {
    let metadata = load_audio_metadata(path)?;
    Ok(validate_audio_metadata_record(&metadata))
}

pub fn list_recordings(
    root: impl AsRef<Path>,
) -> Result<Vec<RecordingSummary>, AudioMetadataError> {
    let root = root.as_ref();
    if !root.exists() {
        return Err(AudioMetadataError::DirectoryNotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(AudioMetadataError::NotDirectory(root.to_path_buf()));
    }

    let entries = fs::read_dir(root).map_err(|source| AudioMetadataError::ReadDirectoryFailed {
        path: root.to_path_buf(),
        source,
    })?;

    let mut recordings = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| AudioMetadataError::ReadDirectoryFailed {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        if let Ok(metadata) = load_audio_metadata(&path) {
            recordings.push(RecordingSummary {
                id: metadata.recording.id,
                title: metadata.recording.title,
                path,
            });
        }
    }

    recordings.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(recordings)
}

pub fn attach_recording(
    options: &AttachRecordingOptions,
) -> Result<AttachedRecording, AudioMetadataError> {
    if !options.project_root.exists() {
        return Err(AudioMetadataError::ProjectNotFound(
            options.project_root.clone(),
        ));
    }
    if !options.project_root.is_dir() {
        return Err(AudioMetadataError::ProjectNotDirectory(
            options.project_root.clone(),
        ));
    }

    validate_recording_id_for_write(&options.id)?;
    validate_audio_path_for_write("audio.metadata", &options.metadata_root)?;
    validate_audio_path_for_write("recording.master", &options.master)?;
    validate_audio_path_for_write("recording.published", &options.published)?;
    validate_audio_path_for_write("publication.feed", &options.feed)?;

    for path in [&options.master, &options.published] {
        if !options.project_root.join(path).is_file() {
            return Err(AudioMetadataError::AudioFileMissing { path: path.clone() });
        }
    }

    let metadata_relative_path = Path::new(&options.metadata_root)
        .join(format!("{}.toml", options.id))
        .to_string_lossy()
        .to_string();
    let metadata_dir = options.project_root.join(&options.metadata_root);
    let metadata_path = options.project_root.join(&metadata_relative_path);
    if metadata_path.exists() {
        return Err(AudioMetadataError::MetadataAlreadyExists(metadata_path));
    }

    fs::create_dir_all(&metadata_dir).map_err(|source| {
        AudioMetadataError::CreateDirectoryFailed {
            path: metadata_dir.clone(),
            source,
        }
    })?;

    let temp_metadata_path = metadata_path.with_file_name(format!("{}.toml.tmp", options.id));
    fs::write(&temp_metadata_path, render_attached_recording(options)).map_err(|source| {
        AudioMetadataError::WriteFileFailed {
            path: temp_metadata_path.clone(),
            source,
        }
    })?;
    fs::rename(&temp_metadata_path, &metadata_path).map_err(|source| {
        AudioMetadataError::InstallMetadataFailed {
            path: metadata_path.clone(),
            source,
        }
    })?;

    Ok(AttachedRecording {
        id: options.id.clone(),
        title: options.title.clone(),
        metadata_path,
        metadata_relative_path,
        master: options.master.clone(),
        published: options.published.clone(),
        feed: options.feed.clone(),
        entry_id: options.entry_id.clone(),
        mime_type: options.mime_type.clone(),
    })
}

pub fn validate_audio_metadata_record(metadata: &AudioMetadata) -> ValidationReport {
    let mut issues = Vec::new();
    validate_id(&mut issues, "recording.id", &metadata.recording.id);

    if metadata.recording.title.trim().is_empty() {
        issues.push(error(
            "missing_recording_title",
            "recording title is required".to_string(),
            Some("recording.title"),
        ));
    }

    validate_relative_path(&mut issues, "recording.master", &metadata.recording.master);

    if let Some(published) = &metadata.recording.published {
        validate_relative_path(&mut issues, "recording.published", published);
    }

    if let Some(channels) = metadata.recording.channels {
        if channels == 0 {
            issues.push(error(
                "invalid_channel_count",
                "channel count must be greater than zero".to_string(),
                Some("recording.channels"),
            ));
        }
    }

    if let Some(sample_rate) = metadata.recording.sample_rate {
        if sample_rate == 0 {
            issues.push(error(
                "invalid_sample_rate",
                "sample rate must be greater than zero".to_string(),
                Some("recording.sample_rate"),
            ));
        }
    }

    if let Some(publication) = &metadata.publication {
        if let Some(feed) = &publication.feed {
            validate_relative_path(&mut issues, "publication.feed", feed);
        }
        if let Some(mime_type) = &publication.mime_type {
            if !mime_type.contains('/') {
                issues.push(warning(
                    "unusual_mime_type",
                    format!("MIME type does not contain '/': {mime_type}"),
                    Some("publication.mime_type"),
                ));
            }
        }
    }

    ValidationReport::from_issues(issues)
}

fn read_metadata(path: &Path) -> Result<String, AudioMetadataError> {
    if !path.exists() {
        return Err(AudioMetadataError::NotFound(path.to_path_buf()));
    }
    if !path.is_file() {
        return Err(AudioMetadataError::NotFile(path.to_path_buf()));
    }
    fs::read_to_string(path).map_err(|source| AudioMetadataError::Unreadable {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_id(issues: &mut Vec<ValidationIssue>, field: &'static str, value: &str) {
    if !recording_id_is_valid(value) {
        issues.push(error(
            "invalid_recording_id",
            format!("{field} must contain only ASCII letters, digits, '-' or '_'"),
            Some(field),
        ));
    }
}

fn validate_relative_path(issues: &mut Vec<ValidationIssue>, field: &'static str, value: &str) {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        issues.push(error(
            "invalid_audio_path",
            format!("{field} must be a relative path inside the project audio metadata boundary"),
            Some(field),
        ));
    }
}

fn validate_recording_id_for_write(value: &str) -> Result<(), AudioMetadataError> {
    if recording_id_is_valid(value) {
        Ok(())
    } else {
        Err(AudioMetadataError::InvalidRecordingId(value.to_string()))
    }
}

fn recording_id_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn validate_audio_path_for_write(field: &str, value: &str) -> Result<(), AudioMetadataError> {
    let path = Path::new(value);
    let invalid = path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir));

    if invalid {
        Err(AudioMetadataError::InvalidAudioPath {
            field: field.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(())
    }
}

fn render_attached_recording(options: &AttachRecordingOptions) -> String {
    let mut metadata = String::new();
    metadata.push_str("[recording]\n");
    metadata.push_str(&format!("id = \"{}\"\n", toml_escape(&options.id)));
    metadata.push_str(&format!("title = \"{}\"\n", toml_escape(&options.title)));
    metadata.push_str(&format!("master = \"{}\"\n", toml_escape(&options.master)));
    metadata.push_str(&format!(
        "published = \"{}\"\n\n",
        toml_escape(&options.published)
    ));
    metadata.push_str("[publication]\n");
    metadata.push_str(&format!("feed = \"{}\"\n", toml_escape(&options.feed)));
    metadata.push_str(&format!(
        "entry_id = \"{}\"\n",
        toml_escape(&options.entry_id)
    ));
    metadata.push_str(&format!(
        "mime_type = \"{}\"\n",
        toml_escape(&options.mime_type)
    ));
    metadata
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn error(code: &'static str, message: String, path: Option<impl Into<String>>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code,
        message,
        path: path.map(|value| value.into()),
    }
}

fn warning(
    code: &'static str,
    message: String,
    path: Option<impl Into<String>>,
) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        code,
        message,
        path: path.map(|value| value.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    #[test]
    fn validates_audio_metadata_example() {
        let report = validate_audio_metadata(repo_path(
            "examples/projects/audio-capsule.wayproject/audio/metadata/field-note.toml",
        ))
        .expect("metadata example should load");

        assert!(report.valid, "{report:#?}");
    }

    #[test]
    fn lists_recording_metadata() {
        let recordings = list_recordings(repo_path(
            "examples/projects/audio-capsule.wayproject/audio/metadata",
        ))
        .expect("recordings should list");

        assert_eq!(recordings.len(), 1);
        assert_eq!(recordings[0].id, "field-note");
    }

    #[test]
    fn rejects_upward_audio_paths() {
        let report = validate_audio_metadata(repo_path(
            "tests/fixtures/audio/invalid-path/field-note.toml",
        ))
        .expect("invalid metadata should parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "invalid_audio_path"));
    }

    #[test]
    fn attach_rejects_project_escape_paths() {
        let root =
            std::env::temp_dir().join(format!("waystone-audio-metadata-{}", std::process::id()));
        let project = root.join("escape.wayproject");
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published file");

        let error = attach_recording(&AttachRecordingOptions {
            project_root: project.clone(),
            metadata_root: "../outside".to_string(),
            id: "note".to_string(),
            title: "Note".to_string(),
            master: "audio/masters/note.flac".to_string(),
            published: "audio/published/note.opus".to_string(),
            feed: "feeds/feed.xml".to_string(),
            entry_id: "tag:example.invalid,2026:note".to_string(),
            mime_type: "audio/ogg; codecs=opus".to_string(),
        })
        .expect_err("escaping metadata path should fail");

        assert!(matches!(
            error,
            AudioMetadataError::InvalidAudioPath { field, .. } if field == "audio.metadata"
        ));

        let _ = fs::remove_dir_all(root);
    }
}
