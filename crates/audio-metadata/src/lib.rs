use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
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

#[derive(Debug, Deserialize)]
pub struct FeedEntryMetadata {
    pub entry: Option<FeedEntrySection>,
    pub enclosure: Option<EnclosureSection>,
}

#[derive(Debug, Deserialize)]
pub struct FeedEntrySection {
    pub id: Option<String>,
    pub title: Option<String>,
    pub updated: Option<String>,
    pub summary: Option<String>,
    pub feed: Option<String>,
    pub recording: Option<String>,
    pub recording_metadata: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnclosureSection {
    pub path: Option<String>,
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

#[derive(Debug, Clone)]
pub struct UpdateRecordingOptions {
    pub project_root: PathBuf,
    pub recording_metadata_path: PathBuf,
    pub title: String,
    pub master: String,
    pub published: String,
    pub feed: String,
    pub entry_id: String,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct UpdatedRecording {
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

#[derive(Debug, Clone)]
pub struct ExportOpusOptions {
    pub project_root: PathBuf,
    pub master: String,
    pub published: String,
    pub preset: String,
}

#[derive(Debug, Clone)]
pub struct ExportedPublicationCopy {
    pub master: String,
    pub published: String,
    pub output_path: PathBuf,
    pub output_relative_path: String,
    pub preset: String,
    pub mime_type: String,
    pub engine: String,
}

#[derive(Debug, Clone)]
pub struct PrepareFeedEntryOptions {
    pub project_root: PathBuf,
    pub recording_metadata_path: PathBuf,
    pub updated: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct PreparedFeedEntry {
    pub recording_id: String,
    pub title: String,
    pub entry_id: String,
    pub feed: String,
    pub output_path: PathBuf,
    pub output_relative_path: String,
    pub published: String,
    pub mime_type: String,
    pub updated: String,
}

#[derive(Debug, Clone)]
pub struct UpdateFeedEntryOptions {
    pub project_root: PathBuf,
    pub recording_metadata_path: PathBuf,
    pub updated: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct UpdatedFeedEntry {
    pub recording_id: String,
    pub title: String,
    pub entry_id: String,
    pub feed: String,
    pub output_path: PathBuf,
    pub output_relative_path: String,
    pub published: String,
    pub mime_type: String,
    pub updated: String,
}

#[derive(Debug, Clone)]
pub struct ValidatePublicationOptions {
    pub project_root: PathBuf,
    pub recording_metadata_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateFeedEntryOptions {
    pub project_root: PathBuf,
    pub feed_entry_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GenerateFeedOptions {
    pub project_root: PathBuf,
    pub feed_path: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedFeed {
    pub feed_path: PathBuf,
    pub feed_relative_path: String,
    pub entries: usize,
    pub updated: String,
}

struct PreparedFeedEntryRender<'a> {
    metadata: &'a AudioMetadata,
    recording_metadata_path: &'a str,
    feed: &'a str,
    entry_id: &'a str,
    published: &'a str,
    mime_type: &'a str,
    updated: &'a str,
    summary: &'a str,
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

    #[error("recording title is required")]
    MissingRecordingTitle,

    #[error("publication copy already exists: {0}")]
    PublicationCopyAlreadyExists(PathBuf),

    #[error("publication copy path must end with .opus: {0}")]
    InvalidPublicationCopyPath(String),

    #[error("unsupported Opus export preset: {0}")]
    UnsupportedOpusPreset(String),

    #[error("Opus encoder could not be started: {source}")]
    OpusEncoderUnavailable { source: io::Error },

    #[error("Opus encoder failed with status {status:?}: {stderr}")]
    OpusEncoderFailed { status: Option<i32>, stderr: String },

    #[error("publication copy could not be written: {path}: {source}")]
    WritePublicationCopyFailed { path: PathBuf, source: io::Error },

    #[error("publication copy could not be installed: {path}: {source}")]
    InstallPublicationCopyFailed { path: PathBuf, source: io::Error },

    #[error("metadata directory could not be created: {path}: {source}")]
    CreateDirectoryFailed { path: PathBuf, source: io::Error },

    #[error("metadata file could not be written: {path}: {source}")]
    WriteFileFailed { path: PathBuf, source: io::Error },

    #[error("metadata file could not be installed: {path}: {source}")]
    InstallMetadataFailed { path: PathBuf, source: io::Error },

    #[error("recording metadata is missing publication field: {0}")]
    MissingPublicationField(&'static str),

    #[error("feed entry metadata already exists: {0}")]
    FeedEntryAlreadyExists(PathBuf),

    #[error("feed entry metadata could not be written: {path}: {source}")]
    WriteFeedEntryFailed { path: PathBuf, source: io::Error },

    #[error("feed entry metadata could not be installed: {path}: {source}")]
    InstallFeedEntryFailed { path: PathBuf, source: io::Error },

    #[error("feed entry metadata is invalid: {path}: {issues:?}")]
    InvalidFeedEntry { path: PathBuf, issues: Vec<String> },

    #[error("feed XML is invalid: {path}: {issue}")]
    InvalidFeedXml { path: PathBuf, issue: String },

    #[error("feed file could not be written: {path}: {source}")]
    WriteFeedFailed { path: PathBuf, source: io::Error },

    #[error("feed file could not be installed: {path}: {source}")]
    InstallFeedFailed { path: PathBuf, source: io::Error },
}

pub fn load_audio_metadata(path: impl AsRef<Path>) -> Result<AudioMetadata, AudioMetadataError> {
    let text = read_metadata(path.as_ref())?;
    toml::from_str(&text).map_err(AudioMetadataError::ParseFailed)
}

pub fn load_feed_entry_metadata(
    path: impl AsRef<Path>,
) -> Result<FeedEntryMetadata, AudioMetadataError> {
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

pub fn update_recording_metadata(
    options: &UpdateRecordingOptions,
) -> Result<UpdatedRecording, AudioMetadataError> {
    check_project_root(&options.project_root)?;
    let existing = load_audio_metadata(&options.recording_metadata_path)?;
    validate_recording_id_for_write(&existing.recording.id)?;
    if options.title.trim().is_empty() {
        return Err(AudioMetadataError::MissingRecordingTitle);
    }
    validate_audio_path_for_write("recording.master", &options.master)?;
    validate_audio_path_for_write("recording.published", &options.published)?;
    validate_audio_path_for_write("publication.feed", &options.feed)?;
    validate_required_publication_value("publication.entry_id", &options.entry_id)?;
    validate_required_publication_value("publication.mime_type", &options.mime_type)?;

    let metadata_relative_path =
        relative_project_path(&options.project_root, &options.recording_metadata_path).ok_or_else(
            || AudioMetadataError::InvalidAudioPath {
                field: "recording.metadata".to_string(),
                value: options.recording_metadata_path.display().to_string(),
            },
        )?;
    validate_audio_path_for_write("recording.metadata", &metadata_relative_path)?;

    for path in [&options.master, &options.published] {
        if !options.project_root.join(path).is_file() {
            return Err(AudioMetadataError::AudioFileMissing { path: path.clone() });
        }
    }

    let temp_metadata_path = options.recording_metadata_path.with_file_name(format!(
        "{}.tmp",
        options
            .recording_metadata_path
            .file_name()
            .expect("recording metadata path has a file name")
            .to_string_lossy()
    ));
    fs::write(
        &temp_metadata_path,
        render_updated_recording(options, &existing),
    )
    .map_err(|source| AudioMetadataError::WriteFileFailed {
        path: temp_metadata_path.clone(),
        source,
    })?;
    fs::rename(&temp_metadata_path, &options.recording_metadata_path).map_err(|source| {
        AudioMetadataError::InstallMetadataFailed {
            path: options.recording_metadata_path.clone(),
            source,
        }
    })?;

    Ok(UpdatedRecording {
        id: existing.recording.id,
        title: options.title.clone(),
        metadata_path: options.recording_metadata_path.clone(),
        metadata_relative_path,
        master: options.master.clone(),
        published: options.published.clone(),
        feed: options.feed.clone(),
        entry_id: options.entry_id.clone(),
        mime_type: options.mime_type.clone(),
    })
}

pub fn export_opus_publication_copy(
    options: &ExportOpusOptions,
) -> Result<ExportedPublicationCopy, AudioMetadataError> {
    check_project_root(&options.project_root)?;
    validate_audio_path_for_write("recording.master", &options.master)?;
    validate_audio_path_for_write("recording.published", &options.published)?;
    validate_opus_preset(&options.preset)?;

    if !options.published.ends_with(".opus") {
        return Err(AudioMetadataError::InvalidPublicationCopyPath(
            options.published.clone(),
        ));
    }
    if !options.project_root.join(&options.master).is_file() {
        return Err(AudioMetadataError::AudioFileMissing {
            path: options.master.clone(),
        });
    }

    let output_relative_path = options.published.clone();
    let output_path = options.project_root.join(&output_relative_path);
    if output_path.exists() {
        return Err(AudioMetadataError::PublicationCopyAlreadyExists(
            output_path,
        ));
    }

    let output_dir = output_path
        .parent()
        .expect("publication copy output has a parent directory");
    fs::create_dir_all(output_dir).map_err(|source| AudioMetadataError::CreateDirectoryFailed {
        path: output_dir.to_path_buf(),
        source,
    })?;

    let temp_output_path = output_path.with_file_name(format!(
        "{}.tmp",
        output_path
            .file_name()
            .expect("publication copy output has a file name")
            .to_string_lossy()
    ));
    export_opus_with_ffmpeg(options, &temp_output_path)?;
    fs::rename(&temp_output_path, &output_path).map_err(|source| {
        AudioMetadataError::InstallPublicationCopyFailed {
            path: output_path.clone(),
            source,
        }
    })?;

    Ok(ExportedPublicationCopy {
        master: options.master.clone(),
        published: options.published.clone(),
        output_path,
        output_relative_path,
        preset: options.preset.clone(),
        mime_type: "audio/ogg; codecs=opus".to_string(),
        engine: "ffmpeg".to_string(),
    })
}

pub fn prepare_feed_entry(
    options: &PrepareFeedEntryOptions,
) -> Result<PreparedFeedEntry, AudioMetadataError> {
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

    let metadata = load_audio_metadata(&options.recording_metadata_path)?;
    let relative_metadata_path =
        relative_project_path(&options.project_root, &options.recording_metadata_path).ok_or_else(
            || AudioMetadataError::InvalidAudioPath {
                field: "recording.metadata".to_string(),
                value: options.recording_metadata_path.display().to_string(),
            },
        )?;
    validate_audio_path_for_write("recording.metadata", &relative_metadata_path)?;

    let Some(publication) = &metadata.publication else {
        return Err(AudioMetadataError::MissingPublicationField("publication"));
    };
    let Some(feed) = publication.feed.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.feed",
        ));
    };
    let Some(entry_id) = publication.entry_id.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.entry_id",
        ));
    };
    let Some(mime_type) = publication.mime_type.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.mime_type",
        ));
    };
    let Some(published) = metadata.recording.published.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "recording.published",
        ));
    };
    let feed = feed.to_string();
    let entry_id = entry_id.to_string();
    let mime_type = mime_type.to_string();
    let published = published.to_string();

    validate_audio_path_for_write("publication.feed", &feed)?;
    validate_audio_path_for_write("recording.published", &published)?;
    validate_audio_path_for_write("feed-entry.output", "feeds/entries")?;
    if !options.project_root.join(&published).is_file() {
        return Err(AudioMetadataError::AudioFileMissing {
            path: published.clone(),
        });
    }

    let output_relative_path = Path::new("feeds/entries")
        .join(format!("{}.toml", metadata.recording.id))
        .to_string_lossy()
        .to_string();
    let output_path = options.project_root.join(&output_relative_path);
    if output_path.exists() {
        return Err(AudioMetadataError::FeedEntryAlreadyExists(output_path));
    }

    let output_dir = output_path
        .parent()
        .expect("feed entry output has a parent directory");
    fs::create_dir_all(output_dir).map_err(|source| AudioMetadataError::CreateDirectoryFailed {
        path: output_dir.to_path_buf(),
        source,
    })?;

    let temp_output_path =
        output_path.with_file_name(format!("{}.toml.tmp", metadata.recording.id));
    fs::write(
        &temp_output_path,
        render_prepared_feed_entry(&PreparedFeedEntryRender {
            metadata: &metadata,
            recording_metadata_path: &relative_metadata_path,
            feed: &feed,
            entry_id: &entry_id,
            published: &published,
            mime_type: &mime_type,
            updated: &options.updated,
            summary: &options.summary,
        }),
    )
    .map_err(|source| AudioMetadataError::WriteFeedEntryFailed {
        path: temp_output_path.clone(),
        source,
    })?;
    fs::rename(&temp_output_path, &output_path).map_err(|source| {
        AudioMetadataError::InstallFeedEntryFailed {
            path: output_path.clone(),
            source,
        }
    })?;

    Ok(PreparedFeedEntry {
        recording_id: metadata.recording.id,
        title: metadata.recording.title,
        entry_id,
        feed,
        output_path,
        output_relative_path,
        published,
        mime_type,
        updated: options.updated.clone(),
    })
}

pub fn update_feed_entry(
    options: &UpdateFeedEntryOptions,
) -> Result<UpdatedFeedEntry, AudioMetadataError> {
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

    let metadata = load_audio_metadata(&options.recording_metadata_path)?;
    let relative_metadata_path =
        relative_project_path(&options.project_root, &options.recording_metadata_path).ok_or_else(
            || AudioMetadataError::InvalidAudioPath {
                field: "recording.metadata".to_string(),
                value: options.recording_metadata_path.display().to_string(),
            },
        )?;
    validate_audio_path_for_write("recording.metadata", &relative_metadata_path)?;

    let Some(publication) = &metadata.publication else {
        return Err(AudioMetadataError::MissingPublicationField("publication"));
    };
    let Some(feed) = publication.feed.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.feed",
        ));
    };
    let Some(entry_id) = publication.entry_id.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.entry_id",
        ));
    };
    let Some(mime_type) = publication.mime_type.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "publication.mime_type",
        ));
    };
    let Some(published) = metadata.recording.published.as_deref() else {
        return Err(AudioMetadataError::MissingPublicationField(
            "recording.published",
        ));
    };
    let feed = feed.to_string();
    let entry_id = entry_id.to_string();
    let mime_type = mime_type.to_string();
    let published = published.to_string();

    validate_audio_path_for_write("publication.feed", &feed)?;
    validate_audio_path_for_write("recording.published", &published)?;
    validate_audio_path_for_write("feed-entry.output", "feeds/entries")?;
    if !options.project_root.join(&published).is_file() {
        return Err(AudioMetadataError::AudioFileMissing {
            path: published.clone(),
        });
    }

    let output_relative_path = Path::new("feeds/entries")
        .join(format!("{}.toml", metadata.recording.id))
        .to_string_lossy()
        .to_string();
    let output_path = options.project_root.join(&output_relative_path);
    if !output_path.exists() {
        return Err(AudioMetadataError::NotFound(output_path));
    }
    if !output_path.is_file() {
        return Err(AudioMetadataError::NotFile(output_path));
    }

    let temp_output_path =
        output_path.with_file_name(format!("{}.toml.tmp", metadata.recording.id));
    fs::write(
        &temp_output_path,
        render_prepared_feed_entry(&PreparedFeedEntryRender {
            metadata: &metadata,
            recording_metadata_path: &relative_metadata_path,
            feed: &feed,
            entry_id: &entry_id,
            published: &published,
            mime_type: &mime_type,
            updated: &options.updated,
            summary: &options.summary,
        }),
    )
    .map_err(|source| AudioMetadataError::WriteFeedEntryFailed {
        path: temp_output_path.clone(),
        source,
    })?;
    fs::rename(&temp_output_path, &output_path).map_err(|source| {
        AudioMetadataError::InstallFeedEntryFailed {
            path: output_path.clone(),
            source,
        }
    })?;

    Ok(UpdatedFeedEntry {
        recording_id: metadata.recording.id,
        title: metadata.recording.title,
        entry_id,
        feed,
        output_path,
        output_relative_path,
        published,
        mime_type,
        updated: options.updated.clone(),
    })
}

pub fn validate_publication_copy(
    options: &ValidatePublicationOptions,
) -> Result<ValidationReport, AudioMetadataError> {
    check_project_root(&options.project_root)?;
    let metadata = load_audio_metadata(&options.recording_metadata_path)?;
    let mut issues = validation_report_into_issues(validate_audio_metadata_record(&metadata));

    match relative_project_path(&options.project_root, &options.recording_metadata_path) {
        Some(relative_metadata_path) => {
            validate_relative_path(&mut issues, "recording.metadata", &relative_metadata_path);
        }
        None => issues.push(error(
            "invalid_audio_path",
            "recording.metadata must be inside the project".to_string(),
            Some("recording.metadata"),
        )),
    }

    validate_existing_project_file(
        &mut issues,
        &options.project_root,
        "recording.master",
        &metadata.recording.master,
        "missing_master_audio",
    );

    if let Some(published) = &metadata.recording.published {
        validate_existing_project_file(
            &mut issues,
            &options.project_root,
            "recording.published",
            published,
            "missing_published_audio",
        );
    } else {
        issues.push(error(
            "missing_publication_field",
            "recording.published is required for publication validation".to_string(),
            Some("recording.published"),
        ));
    }

    if let Some(publication) = &metadata.publication {
        validate_required_publication_field(&mut issues, "publication.feed", &publication.feed);
        validate_required_publication_field(
            &mut issues,
            "publication.entry_id",
            &publication.entry_id,
        );
        validate_required_publication_field(
            &mut issues,
            "publication.mime_type",
            &publication.mime_type,
        );
    } else {
        issues.push(error(
            "missing_publication",
            "publication section is required for publication validation".to_string(),
            Some("publication"),
        ));
    }

    Ok(ValidationReport::from_issues(issues))
}

pub fn validate_feed_entry(
    options: &ValidateFeedEntryOptions,
) -> Result<ValidationReport, AudioMetadataError> {
    check_project_root(&options.project_root)?;
    let feed_entry = load_feed_entry_metadata(&options.feed_entry_path)?;
    let mut issues = Vec::new();

    match relative_project_path(&options.project_root, &options.feed_entry_path) {
        Some(relative_path) => {
            validate_relative_path(&mut issues, "feed-entry.metadata", &relative_path)
        }
        None => issues.push(error(
            "invalid_audio_path",
            "feed-entry.metadata must be inside the project".to_string(),
            Some("feed-entry.metadata"),
        )),
    }

    let Some(entry) = &feed_entry.entry else {
        issues.push(error(
            "missing_feed_entry_section",
            "entry section is required".to_string(),
            Some("entry"),
        ));
        return Ok(ValidationReport::from_issues(issues));
    };
    let Some(enclosure) = &feed_entry.enclosure else {
        issues.push(error(
            "missing_feed_entry_section",
            "enclosure section is required".to_string(),
            Some("enclosure"),
        ));
        return Ok(ValidationReport::from_issues(issues));
    };

    validate_required_feed_entry_field(&mut issues, "entry.id", &entry.id);
    validate_required_feed_entry_field(&mut issues, "entry.title", &entry.title);
    validate_required_feed_entry_field(&mut issues, "entry.updated", &entry.updated);
    validate_required_feed_entry_field(&mut issues, "entry.summary", &entry.summary);
    validate_required_feed_entry_field(&mut issues, "entry.feed", &entry.feed);
    validate_required_feed_entry_field(&mut issues, "entry.recording", &entry.recording);
    validate_required_feed_entry_field(
        &mut issues,
        "entry.recording_metadata",
        &entry.recording_metadata,
    );
    validate_required_feed_entry_field(&mut issues, "enclosure.path", &enclosure.path);
    validate_required_feed_entry_field(&mut issues, "enclosure.mime_type", &enclosure.mime_type);

    if let Some(recording) = &entry.recording {
        validate_id(&mut issues, "entry.recording", recording);
    }
    if let Some(feed) = &entry.feed {
        validate_relative_path(&mut issues, "entry.feed", feed);
    }
    if let Some(recording_metadata) = &entry.recording_metadata {
        validate_relative_path(&mut issues, "entry.recording_metadata", recording_metadata);
    }
    if let Some(path) = &enclosure.path {
        validate_existing_project_file(
            &mut issues,
            &options.project_root,
            "enclosure.path",
            path,
            "missing_enclosure_audio",
        );
    }
    if let Some(mime_type) = &enclosure.mime_type {
        if !mime_type.contains('/') {
            issues.push(warning(
                "unusual_mime_type",
                format!("MIME type does not contain '/': {mime_type}"),
                Some("enclosure.mime_type"),
            ));
        }
    }

    if let Some(recording_metadata) = &entry.recording_metadata {
        if path_is_relative_inside(recording_metadata) {
            let recording_metadata_path = options.project_root.join(recording_metadata);
            if recording_metadata_path.is_file() {
                let recording = load_audio_metadata(&recording_metadata_path)?;
                issues.extend(validation_report_into_issues(validate_publication_copy(
                    &ValidatePublicationOptions {
                        project_root: options.project_root.clone(),
                        recording_metadata_path,
                    },
                )?));
                validate_feed_entry_matches_recording(&mut issues, entry, enclosure, &recording);
            } else {
                issues.push(error(
                    "missing_recording_metadata",
                    format!("entry.recording_metadata does not exist: {recording_metadata}"),
                    Some("entry.recording_metadata"),
                ));
            }
        }
    }

    validate_duplicate_feed_entry_ids(
        &mut issues,
        &options.project_root,
        &options.feed_entry_path,
        entry.id.as_deref(),
    )?;

    Ok(ValidationReport::from_issues(issues))
}

pub fn generate_feed(options: &GenerateFeedOptions) -> Result<GeneratedFeed, AudioMetadataError> {
    check_project_root(&options.project_root)?;
    validate_audio_path_for_write("feed.path", &options.feed_path)?;

    let entries_root = options.project_root.join("feeds/entries");
    let entries =
        load_valid_feed_entries(&options.project_root, &entries_root, &options.feed_path)?;
    let feed_relative_path = options.feed_path.clone();
    let feed_path = options.project_root.join(&feed_relative_path);
    let preserved_entries = load_preserved_atom_entries(&feed_path, &entries)?;
    let updated = feed_updated(&entries, &preserved_entries);
    let feed_dir = feed_path
        .parent()
        .expect("feed output path has a parent directory");

    fs::create_dir_all(feed_dir).map_err(|source| AudioMetadataError::CreateDirectoryFailed {
        path: feed_dir.to_path_buf(),
        source,
    })?;

    let temp_feed_path = feed_path.with_file_name(format!(
        "{}.tmp",
        feed_path
            .file_name()
            .expect("feed output path has a file name")
            .to_string_lossy()
    ));
    fs::write(
        &temp_feed_path,
        render_atom_feed(
            &options.title,
            &feed_relative_path,
            &updated,
            &entries,
            &preserved_entries,
        ),
    )
    .map_err(|source| AudioMetadataError::WriteFeedFailed {
        path: temp_feed_path.clone(),
        source,
    })?;
    fs::rename(&temp_feed_path, &feed_path).map_err(|source| {
        AudioMetadataError::InstallFeedFailed {
            path: feed_path.clone(),
            source,
        }
    })?;

    Ok(GeneratedFeed {
        feed_path,
        feed_relative_path,
        entries: entries.len() + preserved_entries.len(),
        updated,
    })
}

pub fn validate_audio_metadata_record(metadata: &AudioMetadata) -> ValidationReport {
    ValidationReport::from_issues(audio_metadata_issues(metadata))
}

fn audio_metadata_issues(metadata: &AudioMetadata) -> Vec<ValidationIssue> {
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

    issues
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

fn check_project_root(project_root: &Path) -> Result<(), AudioMetadataError> {
    if !project_root.exists() {
        return Err(AudioMetadataError::ProjectNotFound(
            project_root.to_path_buf(),
        ));
    }
    if !project_root.is_dir() {
        return Err(AudioMetadataError::ProjectNotDirectory(
            project_root.to_path_buf(),
        ));
    }
    Ok(())
}

fn validation_report_into_issues(report: ValidationReport) -> Vec<ValidationIssue> {
    report.errors.into_iter().chain(report.warnings).collect()
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

fn validate_required_publication_field(
    issues: &mut Vec<ValidationIssue>,
    field: &'static str,
    value: &Option<String>,
) {
    if value.as_deref().is_none_or(|value| value.trim().is_empty()) {
        issues.push(error(
            "missing_publication_field",
            format!("{field} is required for publication validation"),
            Some(field),
        ));
    }
}

fn validate_required_publication_value(
    field: &'static str,
    value: &str,
) -> Result<(), AudioMetadataError> {
    if value.trim().is_empty() {
        Err(AudioMetadataError::MissingPublicationField(field))
    } else {
        Ok(())
    }
}

fn validate_required_feed_entry_field(
    issues: &mut Vec<ValidationIssue>,
    field: &'static str,
    value: &Option<String>,
) {
    if value.as_deref().is_none_or(|value| value.trim().is_empty()) {
        issues.push(error(
            "missing_feed_entry_field",
            format!("{field} is required for feed-entry validation"),
            Some(field),
        ));
    }
}

fn validate_relative_path(issues: &mut Vec<ValidationIssue>, field: &'static str, value: &str) {
    if !path_is_relative_inside(value) {
        issues.push(error(
            "invalid_audio_path",
            format!("{field} must be a relative path inside the project audio metadata boundary"),
            Some(field),
        ));
    }
}

fn validate_existing_project_file(
    issues: &mut Vec<ValidationIssue>,
    project_root: &Path,
    field: &'static str,
    value: &str,
    missing_code: &'static str,
) {
    validate_relative_path(issues, field, value);
    if path_is_relative_inside(value) && !project_root.join(value).is_file() {
        issues.push(error(
            missing_code,
            format!("{field} does not exist: {value}"),
            Some(field),
        ));
    }
}

fn validate_feed_entry_matches_recording(
    issues: &mut Vec<ValidationIssue>,
    entry: &FeedEntrySection,
    enclosure: &EnclosureSection,
    recording: &AudioMetadata,
) {
    if let Some(recording_id) = &entry.recording {
        if recording_id != &recording.recording.id {
            issues.push(error(
                "feed_entry_recording_mismatch",
                format!(
                    "entry.recording does not match recording.id: {recording_id} != {}",
                    recording.recording.id
                ),
                Some("entry.recording"),
            ));
        }
    }

    if let Some(title) = &entry.title {
        if title != &recording.recording.title {
            issues.push(warning(
                "feed_entry_title_mismatch",
                "entry.title differs from the recording title".to_string(),
                Some("entry.title"),
            ));
        }
    }

    let Some(publication) = &recording.publication else {
        return;
    };

    compare_optional_field(
        issues,
        "entry.id",
        entry.id.as_deref(),
        publication.entry_id.as_deref(),
        "feed_entry_id_mismatch",
    );
    compare_optional_field(
        issues,
        "entry.feed",
        entry.feed.as_deref(),
        publication.feed.as_deref(),
        "feed_entry_feed_mismatch",
    );
    compare_optional_field(
        issues,
        "enclosure.path",
        enclosure.path.as_deref(),
        recording.recording.published.as_deref(),
        "feed_entry_enclosure_mismatch",
    );
    compare_optional_field(
        issues,
        "enclosure.mime_type",
        enclosure.mime_type.as_deref(),
        publication.mime_type.as_deref(),
        "feed_entry_mime_type_mismatch",
    );
}

fn compare_optional_field(
    issues: &mut Vec<ValidationIssue>,
    field: &'static str,
    actual: Option<&str>,
    expected: Option<&str>,
    code: &'static str,
) {
    if let (Some(actual), Some(expected)) = (actual, expected) {
        if actual != expected {
            issues.push(error(
                code,
                format!("{field} does not match recording metadata: {actual} != {expected}"),
                Some(field),
            ));
        }
    }
}

fn validate_duplicate_feed_entry_ids(
    issues: &mut Vec<ValidationIssue>,
    project_root: &Path,
    feed_entry_path: &Path,
    entry_id: Option<&str>,
) -> Result<(), AudioMetadataError> {
    let Some(entry_id) = entry_id.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };

    let feed_entries_root = project_root.join("feeds/entries");
    if !feed_entries_root.is_dir() {
        return Ok(());
    }

    let current_path = absolute_path(feed_entry_path)?;
    for entry in fs::read_dir(&feed_entries_root).map_err(|source| {
        AudioMetadataError::ReadDirectoryFailed {
            path: feed_entries_root.clone(),
            source,
        }
    })? {
        let entry = entry.map_err(|source| AudioMetadataError::ReadDirectoryFailed {
            path: feed_entries_root.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }
        if absolute_path(&path)? == current_path {
            continue;
        }
        if let Ok(metadata) = load_feed_entry_metadata(&path) {
            if metadata.entry.and_then(|entry| entry.id).as_deref() == Some(entry_id) {
                issues.push(error(
                    "duplicate_feed_entry_id",
                    format!("feed entry id is already used by {}", path.display()),
                    Some("entry.id"),
                ));
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct FeedXmlEntry {
    id: String,
    title: String,
    updated: String,
    summary: String,
    feed: String,
    enclosure_path: String,
    enclosure_mime_type: String,
}

#[derive(Debug)]
struct PreservedAtomEntry {
    id: Option<String>,
    updated: Option<String>,
    xml: String,
}

fn load_valid_feed_entries(
    project_root: &Path,
    entries_root: &Path,
    feed_path: &str,
) -> Result<Vec<FeedXmlEntry>, AudioMetadataError> {
    if !entries_root.exists() {
        return Ok(Vec::new());
    }
    if !entries_root.is_dir() {
        return Err(AudioMetadataError::NotDirectory(entries_root.to_path_buf()));
    }

    let mut entries = Vec::new();
    for entry in
        fs::read_dir(entries_root).map_err(|source| AudioMetadataError::ReadDirectoryFailed {
            path: entries_root.to_path_buf(),
            source,
        })?
    {
        let entry = entry.map_err(|source| AudioMetadataError::ReadDirectoryFailed {
            path: entries_root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        let report = validate_feed_entry(&ValidateFeedEntryOptions {
            project_root: project_root.to_path_buf(),
            feed_entry_path: path.clone(),
        })?;
        if !report.valid {
            return Err(AudioMetadataError::InvalidFeedEntry {
                path,
                issues: validation_report_into_issues(report)
                    .into_iter()
                    .map(|issue| format!("{}: {}", issue.code, issue.message))
                    .collect(),
            });
        }

        let entry = feed_xml_entry_from_metadata(&path, &load_feed_entry_metadata(&path)?)?;
        if entry.feed != feed_path {
            return Err(AudioMetadataError::InvalidFeedEntry {
                path,
                issues: vec![format!(
                    "entry.feed does not match configured feed path: {} != {feed_path}",
                    entry.feed
                )],
            });
        }
        entries.push(entry);
    }

    entries.sort_by(|left, right| {
        right
            .updated
            .cmp(&left.updated)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(entries)
}

fn load_preserved_atom_entries(
    feed_path: &Path,
    managed_entries: &[FeedXmlEntry],
) -> Result<Vec<PreservedAtomEntry>, AudioMetadataError> {
    if !feed_path.exists() {
        return Ok(Vec::new());
    }
    if !feed_path.is_file() {
        return Err(AudioMetadataError::NotFile(feed_path.to_path_buf()));
    }

    let feed = fs::read_to_string(feed_path).map_err(|source| AudioMetadataError::Unreadable {
        path: feed_path.to_path_buf(),
        source,
    })?;
    if !feed.contains("<feed") || !feed.contains("</feed>") {
        if feed.contains("WaystoneOS feed placeholder") {
            return Ok(Vec::new());
        }
        return Err(AudioMetadataError::InvalidFeedXml {
            path: feed_path.to_path_buf(),
            issue: "existing feed is not a complete Atom feed".to_string(),
        });
    }

    let existing_entries = extract_atom_entries(feed_path, &feed)?;
    Ok(existing_entries
        .into_iter()
        .filter(|entry| {
            entry.id.as_deref().is_none_or(|id| {
                !managed_entries
                    .iter()
                    .any(|managed_entry| managed_entry.id == id)
            })
        })
        .collect())
}

fn extract_atom_entries(
    feed_path: &Path,
    feed: &str,
) -> Result<Vec<PreservedAtomEntry>, AudioMetadataError> {
    let mut entries = Vec::new();
    let mut offset = 0;

    while let Some(relative_start) = feed[offset..].find("<entry") {
        let start = offset + relative_start;
        let Some(relative_open_end) = feed[start..].find('>') else {
            return Err(AudioMetadataError::InvalidFeedXml {
                path: feed_path.to_path_buf(),
                issue: "entry start tag is incomplete".to_string(),
            });
        };
        let open_end = start + relative_open_end + 1;
        let Some(relative_close_start) = feed[open_end..].find("</entry>") else {
            return Err(AudioMetadataError::InvalidFeedXml {
                path: feed_path.to_path_buf(),
                issue: "entry end tag is missing".to_string(),
            });
        };
        let end = open_end + relative_close_start + "</entry>".len();
        let xml = feed[start..end].trim().to_string();
        entries.push(PreservedAtomEntry {
            id: extract_xml_text(&xml, "id").map(|value| xml_unescape(&value)),
            updated: extract_xml_text(&xml, "updated").map(|value| xml_unescape(&value)),
            xml,
        });
        offset = end;
    }

    Ok(entries)
}

fn extract_xml_text(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

fn feed_updated(
    managed_entries: &[FeedXmlEntry],
    preserved_entries: &[PreservedAtomEntry],
) -> String {
    managed_entries
        .iter()
        .map(|entry| entry.updated.as_str())
        .chain(
            preserved_entries
                .iter()
                .filter_map(|entry| entry.updated.as_deref()),
        )
        .max()
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_string()
}

fn feed_xml_entry_from_metadata(
    path: &Path,
    metadata: &FeedEntryMetadata,
) -> Result<FeedXmlEntry, AudioMetadataError> {
    let entry = metadata
        .entry
        .as_ref()
        .ok_or_else(|| invalid_feed_entry(path, "entry section is required"))?;
    let enclosure = metadata
        .enclosure
        .as_ref()
        .ok_or_else(|| invalid_feed_entry(path, "enclosure section is required"))?;

    Ok(FeedXmlEntry {
        id: required_feed_value(path, "entry.id", &entry.id)?,
        title: required_feed_value(path, "entry.title", &entry.title)?,
        updated: required_feed_value(path, "entry.updated", &entry.updated)?,
        summary: required_feed_value(path, "entry.summary", &entry.summary)?,
        feed: required_feed_value(path, "entry.feed", &entry.feed)?,
        enclosure_path: required_feed_value(path, "enclosure.path", &enclosure.path)?,
        enclosure_mime_type: required_feed_value(
            path,
            "enclosure.mime_type",
            &enclosure.mime_type,
        )?,
    })
}

fn required_feed_value(
    path: &Path,
    field: &str,
    value: &Option<String>,
) -> Result<String, AudioMetadataError> {
    value
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| invalid_feed_entry(path, &format!("{field} is required")))
}

fn invalid_feed_entry(path: &Path, issue: &str) -> AudioMetadataError {
    AudioMetadataError::InvalidFeedEntry {
        path: path.to_path_buf(),
        issues: vec![issue.to_string()],
    }
}

fn validate_recording_id_for_write(value: &str) -> Result<(), AudioMetadataError> {
    if recording_id_is_valid(value) {
        Ok(())
    } else {
        Err(AudioMetadataError::InvalidRecordingId(value.to_string()))
    }
}

fn validate_opus_preset(value: &str) -> Result<(), AudioMetadataError> {
    if matches!(
        value,
        "voice-compact" | "voice-standard" | "spoken-program" | "music-efficient" | "music-quality"
    ) {
        Ok(())
    } else {
        Err(AudioMetadataError::UnsupportedOpusPreset(value.to_string()))
    }
}

fn opus_preset_args(value: &str) -> Result<&'static [&'static str], AudioMetadataError> {
    match value {
        "voice-compact" => Ok(&[
            "-ac",
            "1",
            "-ar",
            "48000",
            "-b:a",
            "32k",
            "-application",
            "voip",
        ]),
        "voice-standard" => Ok(&[
            "-ac",
            "1",
            "-ar",
            "48000",
            "-b:a",
            "48k",
            "-application",
            "voip",
        ]),
        "spoken-program" => Ok(&[
            "-ac",
            "1",
            "-ar",
            "48000",
            "-b:a",
            "64k",
            "-application",
            "audio",
        ]),
        "music-efficient" => Ok(&["-ar", "48000", "-b:a", "96k", "-application", "audio"]),
        "music-quality" => Ok(&["-ar", "48000", "-b:a", "160k", "-application", "audio"]),
        _ => Err(AudioMetadataError::UnsupportedOpusPreset(value.to_string())),
    }
}

fn export_opus_with_ffmpeg(
    options: &ExportOpusOptions,
    temp_output_path: &Path,
) -> Result<(), AudioMetadataError> {
    let master_path = options.project_root.join(&options.master);
    let mut command = Command::new("ffmpeg");
    command.args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"]);
    command.arg(master_path);
    command.args(["-vn", "-c:a", "libopus"]);
    command.args(opus_preset_args(&options.preset)?);
    command.args(["-f", "opus"]);
    command.arg(temp_output_path);

    let output = command
        .output()
        .map_err(|source| AudioMetadataError::OpusEncoderUnavailable { source })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AudioMetadataError::OpusEncoderFailed {
            status: output.status.code(),
            stderr,
        });
    }

    Ok(())
}

fn recording_id_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn path_is_relative_inside(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && !path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn validate_audio_path_for_write(field: &str, value: &str) -> Result<(), AudioMetadataError> {
    if !path_is_relative_inside(value) {
        Err(AudioMetadataError::InvalidAudioPath {
            field: field.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(())
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, AudioMetadataError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|current_dir| current_dir.join(path))
            .map_err(|source| AudioMetadataError::Unreadable {
                path: path.to_path_buf(),
                source,
            })
    }
}

fn relative_project_path(project_root: &Path, path: &Path) -> Option<String> {
    let absolute_metadata_path = absolute_path(path).ok()?;
    let absolute_project = absolute_path(project_root).ok()?;
    let relative = absolute_metadata_path
        .strip_prefix(absolute_project)
        .ok()?
        .to_string_lossy()
        .to_string();
    Some(relative)
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

fn render_updated_recording(options: &UpdateRecordingOptions, existing: &AudioMetadata) -> String {
    let mut metadata = String::new();
    metadata.push_str("[recording]\n");
    metadata.push_str(&format!(
        "id = \"{}\"\n",
        toml_escape(&existing.recording.id)
    ));
    metadata.push_str(&format!("title = \"{}\"\n", toml_escape(&options.title)));
    metadata.push_str(&format!("master = \"{}\"\n", toml_escape(&options.master)));
    metadata.push_str(&format!(
        "published = \"{}\"\n",
        toml_escape(&options.published)
    ));
    if let Some(duration_seconds) = existing.recording.duration_seconds {
        metadata.push_str(&format!("duration_seconds = {duration_seconds}\n"));
    }
    if let Some(channels) = existing.recording.channels {
        metadata.push_str(&format!("channels = {channels}\n"));
    }
    if let Some(sample_rate) = existing.recording.sample_rate {
        metadata.push_str(&format!("sample_rate = {sample_rate}\n"));
    }
    metadata.push('\n');
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

fn render_prepared_feed_entry(fields: &PreparedFeedEntryRender<'_>) -> String {
    let mut entry = String::new();
    entry.push_str("[entry]\n");
    entry.push_str(&format!("id = \"{}\"\n", toml_escape(fields.entry_id)));
    entry.push_str(&format!(
        "title = \"{}\"\n",
        toml_escape(&fields.metadata.recording.title)
    ));
    entry.push_str(&format!("updated = \"{}\"\n", toml_escape(fields.updated)));
    entry.push_str(&format!("summary = \"{}\"\n", toml_escape(fields.summary)));
    entry.push_str(&format!("feed = \"{}\"\n", toml_escape(fields.feed)));
    entry.push_str(&format!(
        "recording = \"{}\"\n",
        toml_escape(&fields.metadata.recording.id)
    ));
    entry.push_str(&format!(
        "recording_metadata = \"{}\"\n\n",
        toml_escape(fields.recording_metadata_path)
    ));
    entry.push_str("[enclosure]\n");
    entry.push_str(&format!("path = \"{}\"\n", toml_escape(fields.published)));
    entry.push_str(&format!(
        "mime_type = \"{}\"\n",
        toml_escape(fields.mime_type)
    ));
    entry
}

fn render_atom_feed(
    title: &str,
    feed_path: &str,
    updated: &str,
    entries: &[FeedXmlEntry],
    preserved_entries: &[PreservedAtomEntry],
) -> String {
    let mut feed = String::new();
    feed.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    feed.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    feed.push_str(&format!("  <title>{}</title>\n", xml_escape(title)));
    feed.push_str(&format!(
        "  <id>{}</id>\n",
        xml_escape(&format!("waystone:feed:{feed_path}"))
    ));
    feed.push_str(&format!("  <updated>{}</updated>\n", xml_escape(updated)));

    for entry in entries {
        feed.push_str(&render_feed_xml_entry(entry));
    }

    for entry in preserved_entries {
        feed.push_str(&entry.xml);
        feed.push('\n');
    }

    feed.push_str("</feed>\n");
    feed
}

fn render_feed_xml_entry(entry: &FeedXmlEntry) -> String {
    let mut xml = String::new();
    xml.push_str("  <entry>\n");
    xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&entry.id)));
    xml.push_str(&format!(
        "    <title>{}</title>\n",
        xml_escape(&entry.title)
    ));
    xml.push_str(&format!(
        "    <updated>{}</updated>\n",
        xml_escape(&entry.updated)
    ));
    xml.push_str(&format!(
        "    <summary>{}</summary>\n",
        xml_escape(&entry.summary)
    ));
    xml.push_str(&format!(
        "    <link rel=\"enclosure\" href=\"{}\" type=\"{}\" />\n",
        xml_escape(&entry.enclosure_path),
        xml_escape(&entry.enclosure_mime_type)
    ));
    xml.push_str("  </entry>\n");
    xml
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
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

    fn ffmpeg_opus_available() -> bool {
        Command::new("ffmpeg")
            .args(["-hide_banner", "-encoders"])
            .output()
            .map(|output| {
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout).contains("libopus")
            })
            .unwrap_or(false)
    }

    fn write_test_wav(path: &Path) {
        let sample_rate = 48_000u32;
        let channels = 1u16;
        let bits_per_sample = 16u16;
        let sample_count = 4_800u32;
        let bytes_per_sample = u32::from(bits_per_sample / 8);
        let data_len = sample_count * u32::from(channels) * bytes_per_sample;
        let byte_rate = sample_rate * u32::from(channels) * bytes_per_sample;
        let block_align = channels * (bits_per_sample / 8);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.resize(bytes.len() + data_len as usize, 0);

        fs::write(path, bytes).expect("test wav file");
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

    #[test]
    fn update_replaces_recording_metadata_and_preserves_measurements() {
        let root = std::env::temp_dir().join(format!(
            "waystone-audio-metadata-update-{}",
            std::process::id()
        ));
        let project = root.join("update.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
        fs::write(project.join("audio/masters/original.flac"), b"master").expect("master file");
        fs::write(
            project.join("audio/masters/updated.flac"),
            b"updated master",
        )
        .expect("updated master file");
        fs::write(project.join("audio/published/original.opus"), b"published")
            .expect("published file");
        fs::write(
            project.join("audio/published/updated.opus"),
            b"updated published",
        )
        .expect("updated published file");
        let metadata_path = project.join("audio/metadata/note.toml");
        fs::write(
            &metadata_path,
            r#"[recording]
id = "note"
title = "Original"
master = "audio/masters/original.flac"
published = "audio/published/original.opus"
duration_seconds = 91
channels = 2
sample_rate = 48000

[publication]
feed = "feeds/original.xml"
entry_id = "tag:example.invalid,2026:original"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("recording metadata");

        let updated = update_recording_metadata(&UpdateRecordingOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path.clone(),
            title: "Updated".to_string(),
            master: "audio/masters/updated.flac".to_string(),
            published: "audio/published/updated.opus".to_string(),
            feed: "feeds/feed.xml".to_string(),
            entry_id: "tag:example.invalid,2026:updated".to_string(),
            mime_type: "audio/ogg; codecs=opus".to_string(),
        })
        .expect("recording metadata should update");

        assert_eq!(updated.id, "note");
        assert_eq!(updated.metadata_relative_path, "audio/metadata/note.toml");
        assert_eq!(updated.metadata_path, metadata_path);

        let metadata = fs::read_to_string(&updated.metadata_path).expect("updated metadata");
        assert!(metadata.contains("id = \"note\""));
        assert!(metadata.contains("title = \"Updated\""));
        assert!(metadata.contains("master = \"audio/masters/updated.flac\""));
        assert!(metadata.contains("published = \"audio/published/updated.opus\""));
        assert!(metadata.contains("duration_seconds = 91"));
        assert!(metadata.contains("channels = 2"));
        assert!(metadata.contains("sample_rate = 48000"));
        assert!(metadata.contains("feed = \"feeds/feed.xml\""));
        assert!(metadata.contains("entry_id = \"tag:example.invalid,2026:updated\""));

        let report = validate_publication_copy(&ValidatePublicationOptions {
            project_root: project,
            recording_metadata_path: updated.metadata_path,
        })
        .expect("updated publication should validate");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_rejects_project_escape_paths() {
        let root = std::env::temp_dir().join(format!(
            "waystone-audio-metadata-update-escape-{}",
            std::process::id()
        ));
        let project = root.join("update-escape.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published file");
        let metadata_path = project.join("audio/metadata/note.toml");
        fs::write(
            &metadata_path,
            r#"[recording]
id = "note"
title = "Note"
master = "audio/masters/note.flac"
published = "audio/published/note.opus"

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:note"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("recording metadata");

        let error = update_recording_metadata(&UpdateRecordingOptions {
            project_root: project,
            recording_metadata_path: metadata_path,
            title: "Updated".to_string(),
            master: "../outside.flac".to_string(),
            published: "audio/published/note.opus".to_string(),
            feed: "feeds/feed.xml".to_string(),
            entry_id: "tag:example.invalid,2026:note".to_string(),
            mime_type: "audio/ogg; codecs=opus".to_string(),
        })
        .expect_err("escaping master path should fail");

        assert!(matches!(
            error,
            AudioMetadataError::InvalidAudioPath { field, .. } if field == "recording.master"
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_opus_writes_encoded_publication_copy() {
        if !ffmpeg_opus_available() {
            eprintln!("skipping real Opus export test because ffmpeg/libopus is unavailable");
            return;
        }

        let root =
            std::env::temp_dir().join(format!("waystone-audio-export-{}", std::process::id()));
        let project = root.join("export.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        write_test_wav(&project.join("audio/masters/note.wav"));

        let exported = export_opus_publication_copy(&ExportOpusOptions {
            project_root: project.clone(),
            master: "audio/masters/note.wav".to_string(),
            published: "audio/published/note.opus".to_string(),
            preset: "voice-standard".to_string(),
        })
        .expect("publication copy should export");

        assert_eq!(exported.output_relative_path, "audio/published/note.opus");
        assert_eq!(exported.mime_type, "audio/ogg; codecs=opus");
        assert_eq!(exported.engine, "ffmpeg");
        assert!(exported.output_path.is_file());
        let output = fs::read(&exported.output_path).expect("encoded output");
        assert!(output.starts_with(b"OggS"));

        let duplicate = export_opus_publication_copy(&ExportOpusOptions {
            project_root: project.clone(),
            master: "audio/masters/note.wav".to_string(),
            published: "audio/published/note.opus".to_string(),
            preset: "voice-standard".to_string(),
        })
        .expect_err("duplicate publication copy should fail");
        assert!(matches!(
            duplicate,
            AudioMetadataError::PublicationCopyAlreadyExists(_)
        ));

        let invalid_preset = export_opus_publication_copy(&ExportOpusOptions {
            project_root: project,
            master: "audio/masters/note.wav".to_string(),
            published: "audio/published/other.opus".to_string(),
            preset: "podcast".to_string(),
        })
        .expect_err("unsupported preset should fail");
        assert!(matches!(
            invalid_preset,
            AudioMetadataError::UnsupportedOpusPreset(_)
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_feed_entry_writes_metadata_sidecar() {
        let root = std::env::temp_dir().join(format!(
            "waystone-audio-metadata-feed-{}",
            std::process::id()
        ));
        let project = root.join("feed.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published file");
        let metadata_path = project.join("audio/metadata/note.toml");
        fs::write(
            &metadata_path,
            r#"[recording]
id = "note"
title = "Note"
master = "audio/masters/note.flac"
published = "audio/published/note.opus"

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:note"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("recording metadata");

        let prepared = prepare_feed_entry(&PrepareFeedEntryOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path.clone(),
            updated: "2026-07-19T00:00:00Z".to_string(),
            summary: "Prepared by metadata test".to_string(),
        })
        .expect("feed entry should prepare");

        assert_eq!(prepared.output_relative_path, "feeds/entries/note.toml");
        assert!(prepared.output_path.is_file());
        let feed_entry = fs::read_to_string(&prepared.output_path).expect("feed entry metadata");
        assert!(feed_entry.contains("[entry]"));
        assert!(feed_entry.contains("recording_metadata = \"audio/metadata/note.toml\""));
        assert!(feed_entry.contains("[enclosure]"));
        assert!(feed_entry.contains("path = \"audio/published/note.opus\""));

        let publication_report = validate_publication_copy(&ValidatePublicationOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path.clone(),
        })
        .expect("publication should validate");
        assert!(publication_report.valid, "{publication_report:#?}");

        let feed_report = validate_feed_entry(&ValidateFeedEntryOptions {
            project_root: project.clone(),
            feed_entry_path: prepared.output_path.clone(),
        })
        .expect("feed entry should validate");
        assert!(feed_report.valid, "{feed_report:#?}");

        fs::write(
            project.join("feeds/feed.xml"),
            r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Old Feed Title</title>
  <id>waystone:feed:feeds/feed.xml</id>
  <updated>2026-07-18T00:00:00Z</updated>
  <entry>
    <id>tag:example.invalid,2026:note</id>
    <title>Old Note Title</title>
    <updated>2026-07-18T00:00:00Z</updated>
    <summary>Old note summary</summary>
  </entry>
  <entry>
    <id>tag:example.invalid,2026:manual</id>
    <title>Manual Entry</title>
    <updated>2026-07-21T00:00:00Z</updated>
    <summary>Preserved manual entry</summary>
  </entry>
</feed>
"#,
        )
        .expect("existing feed XML");

        let generated = generate_feed(&GenerateFeedOptions {
            project_root: project.clone(),
            feed_path: "feeds/feed.xml".to_string(),
            title: "Metadata Feed".to_string(),
        })
        .expect("feed should generate");
        assert_eq!(generated.feed_relative_path, "feeds/feed.xml");
        assert_eq!(generated.entries, 2);
        assert_eq!(generated.updated, "2026-07-21T00:00:00Z");
        assert!(generated.feed_path.is_file());
        let feed_xml = fs::read_to_string(&generated.feed_path).expect("generated feed");
        assert!(feed_xml.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"));
        assert!(feed_xml.contains("<title>Metadata Feed</title>"));
        assert!(feed_xml.contains("<entry>"));
        assert!(feed_xml.contains("<id>tag:example.invalid,2026:note</id>"));
        assert!(!feed_xml.contains("Old Note Title"));
        assert!(feed_xml.contains("<id>tag:example.invalid,2026:manual</id>"));
        assert!(feed_xml.contains("<title>Manual Entry</title>"));
        assert!(feed_xml.contains("<link rel=\"enclosure\" href=\"audio/published/note.opus\""));

        fs::write(
            project.join("feeds/entries/duplicate.toml"),
            r#"[entry]
id = "tag:example.invalid,2026:note"
title = "Duplicate"
updated = "2026-07-19T00:00:00Z"
summary = "Duplicate summary"
feed = "feeds/feed.xml"
recording = "note"
recording_metadata = "audio/metadata/note.toml"

[enclosure]
path = "audio/published/note.opus"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("duplicate feed entry metadata");

        let duplicate_report = validate_feed_entry(&ValidateFeedEntryOptions {
            project_root: project.clone(),
            feed_entry_path: prepared.output_path.clone(),
        })
        .expect("duplicate feed entry should validate as report");
        assert!(!duplicate_report.valid);
        assert!(duplicate_report
            .errors
            .iter()
            .any(|issue| issue.code == "duplicate_feed_entry_id"));

        let duplicate_generate = generate_feed(&GenerateFeedOptions {
            project_root: project.clone(),
            feed_path: "feeds/feed.xml".to_string(),
            title: "Metadata Feed".to_string(),
        })
        .expect_err("duplicate feed entry should block feed generation");
        assert!(matches!(
            duplicate_generate,
            AudioMetadataError::InvalidFeedEntry { .. }
        ));

        let duplicate = prepare_feed_entry(&PrepareFeedEntryOptions {
            project_root: project,
            recording_metadata_path: metadata_path,
            updated: "2026-07-19T00:00:00Z".to_string(),
            summary: "Prepared by metadata test".to_string(),
        })
        .expect_err("duplicate feed entry should fail");
        assert!(matches!(
            duplicate,
            AudioMetadataError::FeedEntryAlreadyExists(_)
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn generate_feed_rejects_malformed_existing_feed_xml() {
        let root = std::env::temp_dir().join(format!(
            "waystone-audio-metadata-feed-invalid-{}",
            std::process::id()
        ));
        let project = root.join("feed-invalid.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("feeds")).expect("feeds directory");
        fs::write(project.join("feeds/feed.xml"), "not atom xml").expect("invalid feed XML");

        let error = generate_feed(&GenerateFeedOptions {
            project_root: project,
            feed_path: "feeds/feed.xml".to_string(),
            title: "Metadata Feed".to_string(),
        })
        .expect_err("malformed existing feed XML should fail");

        assert!(matches!(error, AudioMetadataError::InvalidFeedXml { .. }));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_feed_entry_rewrites_existing_sidecar_from_recording_metadata() {
        let root = std::env::temp_dir().join(format!(
            "waystone-audio-metadata-feed-update-{}",
            std::process::id()
        ));
        let project = root.join("feed-update.wayproject");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master file");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published file");
        fs::write(
            project.join("audio/published/revised.opus"),
            b"revised published",
        )
        .expect("revised published file");
        let metadata_path = project.join("audio/metadata/note.toml");
        fs::write(
            &metadata_path,
            r#"[recording]
id = "note"
title = "Note"
master = "audio/masters/note.flac"
published = "audio/published/note.opus"

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:note"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("recording metadata");

        let prepared = prepare_feed_entry(&PrepareFeedEntryOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path.clone(),
            updated: "2026-07-19T00:00:00Z".to_string(),
            summary: "Original summary".to_string(),
        })
        .expect("feed entry should prepare");
        assert!(prepared.output_path.is_file());

        update_recording_metadata(&UpdateRecordingOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path.clone(),
            title: "Note Revised".to_string(),
            master: "audio/masters/note.flac".to_string(),
            published: "audio/published/revised.opus".to_string(),
            feed: "feeds/feed.xml".to_string(),
            entry_id: "tag:example.invalid,2026:note-revised".to_string(),
            mime_type: "audio/ogg; codecs=opus".to_string(),
        })
        .expect("recording metadata should update");

        let updated = update_feed_entry(&UpdateFeedEntryOptions {
            project_root: project.clone(),
            recording_metadata_path: metadata_path,
            updated: "2026-07-20T00:00:00Z".to_string(),
            summary: "Revised summary".to_string(),
        })
        .expect("feed entry should update");

        assert_eq!(updated.output_relative_path, "feeds/entries/note.toml");
        assert_eq!(updated.title, "Note Revised");
        assert_eq!(updated.entry_id, "tag:example.invalid,2026:note-revised");
        assert_eq!(updated.published, "audio/published/revised.opus");

        let feed_entry = fs::read_to_string(&updated.output_path).expect("updated feed entry");
        assert!(feed_entry.contains("title = \"Note Revised\""));
        assert!(feed_entry.contains("updated = \"2026-07-20T00:00:00Z\""));
        assert!(feed_entry.contains("summary = \"Revised summary\""));
        assert!(feed_entry.contains("id = \"tag:example.invalid,2026:note-revised\""));
        assert!(feed_entry.contains("path = \"audio/published/revised.opus\""));

        let report = validate_feed_entry(&ValidateFeedEntryOptions {
            project_root: project,
            feed_entry_path: updated.output_path,
        })
        .expect("updated feed entry should validate");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }
}
