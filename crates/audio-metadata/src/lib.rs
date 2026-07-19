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
pub struct ValidatePublicationOptions {
    pub project_root: PathBuf,
    pub recording_metadata_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateFeedEntryOptions {
    pub project_root: PathBuf,
    pub feed_entry_path: PathBuf,
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
}
