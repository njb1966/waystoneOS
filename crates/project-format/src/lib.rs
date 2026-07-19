use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub const SUPPORTED_SCHEMA: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub waystone: WaystoneSection,
    pub project: ProjectSection,
    pub content: ContentSection,
    pub audio: Option<AudioSection>,
    pub feed: Option<FeedSection>,
    pub publish: Option<PublishSection>,
}

#[derive(Debug, Deserialize)]
pub struct WaystoneSection {
    pub schema: u32,
    pub created_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub language: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContentSection {
    pub root: String,
    pub index: String,
}

#[derive(Debug, Deserialize)]
pub struct AudioSection {
    pub masters: Option<String>,
    pub published: Option<String>,
    pub metadata: Option<String>,
    pub master_format: Option<String>,
    pub publish_format: Option<String>,
    pub publish_bitrate: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FeedSection {
    pub enabled: Option<bool>,
    #[serde(rename = "type")]
    pub feed_type: Option<String>,
    pub path: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishSection {
    pub targets: Option<Vec<PublishTarget>>,
}

#[derive(Debug, Deserialize)]
pub struct PublishTarget {
    pub name: String,
    pub method: String,
    pub host: Option<String>,
    pub identity: Option<String>,
    pub remote_path: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub delete_policy: Option<String>,
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

#[derive(Debug)]
pub struct ProjectInspection {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub content_root: String,
    pub content_index: String,
    pub publish_targets: Vec<String>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ProjectCreateOptions {
    pub parent: PathBuf,
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub content_index: String,
    pub language: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AddRemovablePublishTargetOptions {
    pub project_root: PathBuf,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct CreatedProject {
    pub project_path: PathBuf,
    pub schema: u32,
}

#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ProjectFormatError {
    #[error("project path does not exist: {0}")]
    ProjectNotFound(PathBuf),

    #[error("project path is not a directory: {0}")]
    ProjectNotDirectory(PathBuf),

    #[error("project manifest is missing: {0}")]
    ManifestMissing(PathBuf),

    #[error("project manifest could not be read: {path}: {source}")]
    ManifestUnreadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("project manifest could not be parsed: {0}")]
    ManifestParseFailed(toml::de::Error),

    #[error("project already exists: {0}")]
    ProjectAlreadyExists(PathBuf),

    #[error("invalid project id: {0}")]
    InvalidProjectId(String),

    #[error("invalid project type: {0}")]
    InvalidProjectType(String),

    #[error("invalid project path in {field}: {value}")]
    InvalidProjectPath { field: String, value: String },

    #[error("project directory could not be created: {path}: {source}")]
    CreateDirectoryFailed { path: PathBuf, source: io::Error },

    #[error("project file could not be written: {path}: {source}")]
    WriteFileFailed { path: PathBuf, source: io::Error },

    #[error("project manifest could not be installed: {path}: {source}")]
    InstallManifestFailed { path: PathBuf, source: io::Error },

    #[error("project directory could not be read: {path}: {source}")]
    ReadDirectoryFailed { path: PathBuf, source: io::Error },

    #[error("duplicate publish target: {0}")]
    DuplicatePublishTarget(String),

    #[error("invalid publish target name: {0}")]
    InvalidPublishTargetName(String),
}

pub fn create_project(
    options: &ProjectCreateOptions,
) -> Result<CreatedProject, ProjectFormatError> {
    validate_project_id(&options.id)?;
    if !supported_project_type(&options.project_type) {
        return Err(ProjectFormatError::InvalidProjectType(
            options.project_type.clone(),
        ));
    }
    validate_portable_path("content_index", &options.content_index)?;

    let project_path = options.parent.join(format!("{}.wayproject", options.id));
    if project_path.exists() {
        return Err(ProjectFormatError::ProjectAlreadyExists(project_path));
    }

    let content_root = project_path.join("content");
    fs::create_dir_all(&content_root).map_err(|source| {
        ProjectFormatError::CreateDirectoryFailed {
            path: content_root.clone(),
            source,
        }
    })?;

    let index_path = content_root.join(&options.content_index);
    let index_text = format!("# {}\n\n", options.name);
    fs::write(&index_path, index_text).map_err(|source| ProjectFormatError::WriteFileFailed {
        path: index_path,
        source,
    })?;

    if audio_capable_project_type(&options.project_type) {
        create_audio_project_defaults(&project_path)?;
    }

    let manifest_path = project_path.join("project.toml");
    let temp_manifest_path = project_path.join("project.toml.tmp");
    fs::write(&temp_manifest_path, render_manifest(options)).map_err(|source| {
        ProjectFormatError::WriteFileFailed {
            path: temp_manifest_path.clone(),
            source,
        }
    })?;
    fs::rename(&temp_manifest_path, &manifest_path).map_err(|source| {
        ProjectFormatError::InstallManifestFailed {
            path: manifest_path,
            source,
        }
    })?;

    Ok(CreatedProject {
        project_path,
        schema: SUPPORTED_SCHEMA,
    })
}

fn create_audio_project_defaults(project_path: &Path) -> Result<(), ProjectFormatError> {
    for relative in [
        "audio/masters",
        "audio/published",
        "audio/metadata",
        "feeds",
    ] {
        let path = project_path.join(relative);
        fs::create_dir_all(&path)
            .map_err(|source| ProjectFormatError::CreateDirectoryFailed { path, source })?;
    }

    let feed_path = project_path.join("feeds/feed.xml");
    fs::write(&feed_path, initial_feed_placeholder()).map_err(|source| {
        ProjectFormatError::WriteFileFailed {
            path: feed_path,
            source,
        }
    })?;

    Ok(())
}

pub fn list_projects(root: impl AsRef<Path>) -> Result<Vec<ProjectSummary>, ProjectFormatError> {
    let root = root.as_ref();
    if !root.exists() {
        return Err(ProjectFormatError::ProjectNotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(ProjectFormatError::ProjectNotDirectory(root.to_path_buf()));
    }

    let mut projects = Vec::new();
    collect_projects_bounded(root, 0, &mut projects)?;
    projects.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(projects)
}

pub fn add_removable_publish_target(
    options: &AddRemovablePublishTargetOptions,
) -> Result<(), ProjectFormatError> {
    validate_publish_target_name(&options.name)?;
    validate_portable_path("publish target path", &options.path)?;

    let manifest = load_manifest(&options.project_root)?;
    if manifest
        .publish
        .as_ref()
        .and_then(|publish| publish.targets.as_ref())
        .is_some_and(|targets| targets.iter().any(|target| target.name == options.name))
    {
        return Err(ProjectFormatError::DuplicatePublishTarget(
            options.name.clone(),
        ));
    }

    let manifest_path = options.project_root.join("project.toml");
    let mut manifest_text = fs::read_to_string(&manifest_path).map_err(|source| {
        ProjectFormatError::ManifestUnreadable {
            path: manifest_path.clone(),
            source,
        }
    })?;
    if !manifest_text.ends_with('\n') {
        manifest_text.push('\n');
    }
    manifest_text.push('\n');
    manifest_text.push_str("[[publish.targets]]\n");
    manifest_text.push_str(&format!("name = \"{}\"\n", toml_escape(&options.name)));
    manifest_text.push_str("method = \"removable\"\n");
    manifest_text.push_str(&format!("path = \"{}\"\n", toml_escape(&options.path)));
    manifest_text.push_str("delete_policy = \"forbid\"\n");

    let temp_manifest_path = options.project_root.join("project.toml.tmp");
    fs::write(&temp_manifest_path, manifest_text).map_err(|source| {
        ProjectFormatError::WriteFileFailed {
            path: temp_manifest_path.clone(),
            source,
        }
    })?;
    fs::rename(&temp_manifest_path, &manifest_path).map_err(|source| {
        ProjectFormatError::InstallManifestFailed {
            path: manifest_path,
            source,
        }
    })?;

    Ok(())
}

pub fn load_manifest(project_root: impl AsRef<Path>) -> Result<Manifest, ProjectFormatError> {
    let project_root = project_root.as_ref();
    if !project_root.exists() {
        return Err(ProjectFormatError::ProjectNotFound(
            project_root.to_path_buf(),
        ));
    }
    if !project_root.is_dir() {
        return Err(ProjectFormatError::ProjectNotDirectory(
            project_root.to_path_buf(),
        ));
    }

    let manifest_path = project_root.join("project.toml");
    if !manifest_path.exists() {
        return Err(ProjectFormatError::ManifestMissing(manifest_path));
    }

    let manifest_text = fs::read_to_string(&manifest_path).map_err(|source| {
        ProjectFormatError::ManifestUnreadable {
            path: manifest_path,
            source,
        }
    })?;

    toml::from_str(&manifest_text).map_err(ProjectFormatError::ManifestParseFailed)
}

pub fn inspect_project(
    project_root: impl AsRef<Path>,
) -> Result<ProjectInspection, ProjectFormatError> {
    let project_root = project_root.as_ref();
    let manifest = load_manifest(project_root)?;
    let report = validate_manifest(project_root, &manifest);
    let publish_targets = manifest
        .publish
        .as_ref()
        .and_then(|publish| publish.targets.as_ref())
        .map(|targets| targets.iter().map(|target| target.name.clone()).collect())
        .unwrap_or_default();

    Ok(ProjectInspection {
        schema: manifest.waystone.schema,
        id: manifest.project.id,
        name: manifest.project.name,
        project_type: manifest.project.project_type,
        content_root: manifest.content.root,
        content_index: manifest.content.index,
        publish_targets,
        warnings: report.warnings,
    })
}

pub fn validate_project(
    project_root: impl AsRef<Path>,
) -> Result<ValidationReport, ProjectFormatError> {
    let project_root = project_root.as_ref();
    let manifest = load_manifest(project_root)?;
    Ok(validate_manifest(project_root, &manifest))
}

fn collect_projects_bounded(
    root: &Path,
    depth: usize,
    projects: &mut Vec<ProjectSummary>,
) -> Result<(), ProjectFormatError> {
    if depth > 1 {
        return Ok(());
    }

    let entries = fs::read_dir(root).map_err(|source| ProjectFormatError::ReadDirectoryFailed {
        path: root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| ProjectFormatError::ReadDirectoryFailed {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".wayproject"))
        {
            if let Ok(inspection) = inspect_project(&path) {
                projects.push(ProjectSummary {
                    schema: inspection.schema,
                    id: inspection.id,
                    name: inspection.name,
                    project_type: inspection.project_type,
                    path,
                });
            }
            continue;
        }

        collect_projects_bounded(&path, depth + 1, projects)?;
    }

    Ok(())
}

pub fn validate_manifest(project_root: &Path, manifest: &Manifest) -> ValidationReport {
    let mut issues = Vec::new();

    if manifest.waystone.schema != SUPPORTED_SCHEMA {
        issues.push(error(
            "unsupported_schema",
            format!(
                "schema {} is not supported; expected {}",
                manifest.waystone.schema, SUPPORTED_SCHEMA
            ),
            Some("waystone.schema"),
        ));
    }

    if !supported_project_type(&manifest.project.project_type) {
        issues.push(error(
            "unsupported_project_type",
            format!(
                "unsupported project type: {}",
                manifest.project.project_type
            ),
            Some("project.type"),
        ));
    }

    validate_local_path(&mut issues, "content.root", &manifest.content.root);
    validate_local_path(&mut issues, "content.index", &manifest.content.index);

    let content_root = project_root.join(&manifest.content.root);
    if !content_root.is_dir() {
        issues.push(error(
            "missing_content_root",
            format!("content root is missing: {}", manifest.content.root),
            Some(manifest.content.root.as_str()),
        ));
    }

    let content_index = content_root.join(&manifest.content.index);
    if !content_index.is_file() {
        issues.push(error(
            "missing_content_index",
            format!(
                "content index is missing: {}/{}",
                manifest.content.root, manifest.content.index
            ),
            Some(format!(
                "{}/{}",
                manifest.content.root, manifest.content.index
            )),
        ));
    }

    if let Some(audio) = &manifest.audio {
        for (field, value) in [
            ("audio.masters", audio.masters.as_deref()),
            ("audio.published", audio.published.as_deref()),
            ("audio.metadata", audio.metadata.as_deref()),
        ] {
            if let Some(path) = value {
                validate_local_path(&mut issues, field, path);
            }
        }
    }

    if let Some(feed) = &manifest.feed {
        if let Some(path) = &feed.path {
            validate_local_path(&mut issues, "feed.path", path);
            if feed.enabled.unwrap_or(false) && !project_root.join(path).is_file() {
                issues.push(warning(
                    "missing_feed_file",
                    format!("feed is enabled but file is missing: {path}"),
                    Some(path.as_str()),
                ));
            }
        }
    }

    validate_publish_targets(&mut issues, manifest);

    ValidationReport::from_issues(issues)
}

fn validate_publish_targets(issues: &mut Vec<ValidationIssue>, manifest: &Manifest) {
    let Some(targets) = manifest
        .publish
        .as_ref()
        .and_then(|publish| publish.targets.as_ref())
    else {
        return;
    };

    let mut names = HashSet::new();
    for target in targets {
        if !names.insert(target.name.as_str()) {
            issues.push(error(
                "duplicate_publish_target",
                format!("duplicate publish target: {}", target.name),
                Some(format!("publish.targets.{}", target.name)),
            ));
        }

        if !supported_publish_method(&target.method) {
            issues.push(error(
                "unsupported_publish_method",
                format!("unsupported publish method: {}", target.method),
                Some(format!("publish.targets.{}.method", target.name)),
            ));
        }

        if let Some(path) = &target.path {
            validate_local_path(
                issues,
                &format!("publish.targets.{}.path", target.name),
                path,
            );
        }

        validate_delete_policy(issues, target);
        validate_target_shape(issues, target);
    }
}

fn validate_delete_policy(issues: &mut Vec<ValidationIssue>, target: &PublishTarget) {
    let Some(policy) = &target.delete_policy else {
        return;
    };

    if !matches!(policy.as_str(), "confirm" | "forbid") {
        issues.push(error(
            "unsupported_delete_policy",
            format!(
                "unsupported delete policy for target {}: {}",
                target.name, policy
            ),
            Some(format!("publish.targets.{}.delete_policy", target.name)),
        ));
    }
}

fn validate_target_shape(issues: &mut Vec<ValidationIssue>, target: &PublishTarget) {
    match target.method.as_str() {
        "rsync" | "scp" | "sftp" => {
            if target.host.as_deref().is_none_or(str::is_empty) {
                issues.push(error(
                    "missing_publish_host",
                    format!("{} target {} requires host", target.method, target.name),
                    Some(format!("publish.targets.{}.host", target.name)),
                ));
            }
            if target.remote_path.as_deref().is_none_or(str::is_empty) {
                issues.push(error(
                    "missing_remote_path",
                    format!(
                        "{} target {} requires remote_path",
                        target.method, target.name
                    ),
                    Some(format!("publish.targets.{}.remote_path", target.name)),
                ));
            }
            if target.identity.as_deref().is_none_or(str::is_empty) {
                issues.push(error(
                    "missing_publish_identity",
                    format!("{} target {} requires identity", target.method, target.name),
                    Some(format!("publish.targets.{}.identity", target.name)),
                ));
            }
        }
        "removable" => {
            if target.path.as_deref().is_none_or(str::is_empty) {
                issues.push(error(
                    "missing_export_path",
                    format!("removable target {} requires path", target.name),
                    Some(format!("publish.targets.{}.path", target.name)),
                ));
            }
        }
        _ => {}
    }
}

fn validate_local_path(issues: &mut Vec<ValidationIssue>, field: &str, value: &str) {
    let path = Path::new(value);
    if path.is_absolute() {
        issues.push(error(
            "invalid_project_path",
            format!("{field} must be relative: {value}"),
            Some(field),
        ));
        return;
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        issues.push(error(
            "invalid_project_path",
            format!("{field} must not traverse upward: {value}"),
            Some(field),
        ));
    }
}

fn validate_project_id(id: &str) -> Result<(), ProjectFormatError> {
    let valid = !id.is_empty()
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));

    if valid {
        Ok(())
    } else {
        Err(ProjectFormatError::InvalidProjectId(id.to_string()))
    }
}

fn validate_publish_target_name(name: &str) -> Result<(), ProjectFormatError> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));

    if valid {
        Ok(())
    } else {
        Err(ProjectFormatError::InvalidPublishTargetName(
            name.to_string(),
        ))
    }
}

fn validate_portable_path(field: &str, value: &str) -> Result<(), ProjectFormatError> {
    let path = Path::new(value);
    let invalid = path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir));

    if invalid {
        Err(ProjectFormatError::InvalidProjectPath {
            field: field.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(())
    }
}

fn render_manifest(options: &ProjectCreateOptions) -> String {
    let mut manifest = String::new();
    manifest.push_str("[waystone]\n");
    manifest.push_str(&format!("schema = {}\n", SUPPORTED_SCHEMA));
    manifest.push_str("created_by = \"WaystoneOS\"\n\n");
    manifest.push_str("[project]\n");
    manifest.push_str(&format!("id = \"{}\"\n", toml_escape(&options.id)));
    manifest.push_str(&format!("name = \"{}\"\n", toml_escape(&options.name)));
    manifest.push_str(&format!(
        "type = \"{}\"\n",
        toml_escape(&options.project_type)
    ));
    if let Some(language) = &options.language {
        manifest.push_str(&format!("language = \"{}\"\n", toml_escape(language)));
    }
    if let Some(author) = &options.author {
        manifest.push_str(&format!("author = \"{}\"\n", toml_escape(author)));
    }
    manifest.push_str("\n[content]\n");
    manifest.push_str("root = \"content\"\n");
    manifest.push_str(&format!(
        "index = \"{}\"\n",
        toml_escape(&options.content_index)
    ));
    if audio_capable_project_type(&options.project_type) {
        manifest.push_str("\n[audio]\n");
        manifest.push_str("masters = \"audio/masters\"\n");
        manifest.push_str("published = \"audio/published\"\n");
        manifest.push_str("metadata = \"audio/metadata\"\n");
        manifest.push_str("master_format = \"flac\"\n");
        manifest.push_str("publish_format = \"opus\"\n");
        manifest.push_str("publish_bitrate = 96000\n");
        manifest.push_str("\n[feed]\n");
        manifest.push_str("enabled = true\n");
        manifest.push_str("type = \"atom\"\n");
        manifest.push_str("path = \"feeds/feed.xml\"\n");
        manifest.push_str(&format!("title = \"{}\"\n", toml_escape(&options.name)));
    }
    manifest
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn supported_project_type(project_type: &str) -> bool {
    matches!(
        project_type,
        "capsule"
            | "gemlog"
            | "gopherhole"
            | "spartan-site"
            | "audio-series"
            | "feed"
            | "pubnix-home"
            | "documentation-archive"
            | "classroom-assignment"
            | "mixed-publication"
    )
}

fn audio_capable_project_type(project_type: &str) -> bool {
    matches!(project_type, "audio-series" | "mixed-publication")
}

fn initial_feed_placeholder() -> &'static str {
    "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!-- WaystoneOS feed placeholder; run record generate-feed after preparing entries. -->\n"
}

fn supported_publish_method(method: &str) -> bool {
    matches!(
        method,
        "rsync" | "scp" | "sftp" | "titan" | "git" | "local-service" | "removable"
    )
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

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    #[test]
    fn validates_minimal_capsule_example() {
        let report = validate_project(repo_path("examples/projects/minimal-capsule.wayproject"))
            .expect("minimal capsule fixture should load");
        assert!(report.valid, "{report:#?}");
    }

    #[test]
    fn reports_missing_content_index() {
        let report = validate_project(repo_path(
            "tests/fixtures/projects/invalid-missing-index.wayproject",
        ))
        .expect("invalid fixture should still parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "missing_content_index"));
    }

    #[test]
    fn rejects_path_traversal() {
        let report = validate_project(repo_path(
            "tests/fixtures/projects/invalid-path-traversal.wayproject",
        ))
        .expect("invalid fixture should still parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "invalid_project_path"));
    }

    #[test]
    fn rejects_absolute_paths() {
        let report = validate_project(repo_path(
            "tests/fixtures/projects/invalid-absolute-path.wayproject",
        ))
        .expect("invalid fixture should still parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "invalid_project_path"));
    }

    #[test]
    fn inspects_project_identity() {
        let inspection = inspect_project(repo_path("examples/projects/minimal-capsule.wayproject"))
            .expect("minimal capsule fixture should inspect");

        assert_eq!(inspection.schema, 1);
        assert_eq!(inspection.id, "minimal-capsule");
        assert_eq!(inspection.project_type, "capsule");
    }

    #[test]
    fn creates_minimal_project() {
        let root = unique_temp_root("create-minimal-project");
        fs::create_dir_all(&root).expect("temp root should be created");

        let created = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "created-capsule".to_string(),
            name: "Created Capsule".to_string(),
            project_type: "capsule".to_string(),
            content_index: "index.gmi".to_string(),
            language: Some("en".to_string()),
            author: Some("WaystoneOS".to_string()),
        })
        .expect("project should be created");

        assert_eq!(created.schema, SUPPORTED_SCHEMA);
        assert!(!created.project_path.join("audio").exists());
        assert!(!created.project_path.join("feeds").exists());
        let report = validate_project(&created.project_path).expect("created project should load");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn creates_audio_capable_project_defaults() {
        let root = unique_temp_root("create-audio-project");
        fs::create_dir_all(&root).expect("temp root should be created");

        let created = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "created-audio".to_string(),
            name: "Created Audio".to_string(),
            project_type: "audio-series".to_string(),
            content_index: "index.gmi".to_string(),
            language: Some("en".to_string()),
            author: Some("WaystoneOS".to_string()),
        })
        .expect("audio project should be created");

        assert!(created.project_path.join("audio/masters").is_dir());
        assert!(created.project_path.join("audio/published").is_dir());
        assert!(created.project_path.join("audio/metadata").is_dir());
        assert!(created.project_path.join("feeds").is_dir());
        assert!(created.project_path.join("feeds/feed.xml").is_file());

        let manifest = fs::read_to_string(created.project_path.join("project.toml"))
            .expect("manifest should read");
        assert!(manifest.contains("[audio]"));
        assert!(manifest.contains("metadata = \"audio/metadata\""));
        assert!(manifest.contains("[feed]"));
        assert!(manifest.contains("path = \"feeds/feed.xml\""));

        let report = validate_project(&created.project_path).expect("created project should load");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn creates_mixed_publication_audio_defaults() {
        let root = unique_temp_root("create-mixed-project");
        fs::create_dir_all(&root).expect("temp root should be created");

        let created = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "created-mixed".to_string(),
            name: "Created Mixed".to_string(),
            project_type: "mixed-publication".to_string(),
            content_index: "index.gmi".to_string(),
            language: Some("en".to_string()),
            author: None,
        })
        .expect("mixed project should be created");

        assert!(created.project_path.join("audio/metadata").is_dir());
        assert!(created.project_path.join("feeds/feed.xml").is_file());

        let report = validate_project(&created.project_path).expect("created project should load");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn adds_removable_publish_target() {
        let root = unique_temp_root("add-removable-target");
        fs::create_dir_all(&root).expect("temp root should be created");

        let created = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "target-capsule".to_string(),
            name: "Target Capsule".to_string(),
            project_type: "capsule".to_string(),
            content_index: "index.gmi".to_string(),
            language: Some("en".to_string()),
            author: None,
        })
        .expect("project should be created");

        add_removable_publish_target(&AddRemovablePublishTargetOptions {
            project_root: created.project_path.clone(),
            name: "export".to_string(),
            path: "publish/export".to_string(),
        })
        .expect("target should be added");

        let inspection =
            inspect_project(&created.project_path).expect("created project should inspect");
        assert_eq!(inspection.publish_targets, vec!["export".to_string()]);
        let report = validate_project(&created.project_path).expect("project should validate");
        assert!(report.valid, "{report:#?}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_duplicate_publish_target() {
        let root = unique_temp_root("duplicate-removable-target");
        fs::create_dir_all(&root).expect("temp root should be created");

        let created = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "target-capsule".to_string(),
            name: "Target Capsule".to_string(),
            project_type: "capsule".to_string(),
            content_index: "index.gmi".to_string(),
            language: Some("en".to_string()),
            author: None,
        })
        .expect("project should be created");

        let options = AddRemovablePublishTargetOptions {
            project_root: created.project_path,
            name: "export".to_string(),
            path: "publish/export".to_string(),
        };
        add_removable_publish_target(&options).expect("target should be added");
        let error =
            add_removable_publish_target(&options).expect_err("duplicate target should fail");
        assert!(matches!(
            error,
            ProjectFormatError::DuplicatePublishTarget(_)
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_project_id_with_path_separator() {
        let root = unique_temp_root("invalid-project-id");
        fs::create_dir_all(&root).expect("temp root should be created");

        let error = create_project(&ProjectCreateOptions {
            parent: root.clone(),
            id: "bad/id".to_string(),
            name: "Bad ID".to_string(),
            project_type: "capsule".to_string(),
            content_index: "index.gmi".to_string(),
            language: None,
            author: None,
        })
        .expect_err("project id with slash should fail");

        assert!(matches!(error, ProjectFormatError::InvalidProjectId(_)));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn lists_projects_with_bounded_category_depth() {
        let root = unique_temp_root("list-projects");
        let category = root.join("Capsules");
        fs::create_dir_all(&category).expect("category should be created");

        create_project(&ProjectCreateOptions {
            parent: category,
            id: "listed-capsule".to_string(),
            name: "Listed Capsule".to_string(),
            project_type: "capsule".to_string(),
            content_index: "index.gmi".to_string(),
            language: None,
            author: None,
        })
        .expect("project should be created");

        let projects = list_projects(&root).expect("projects should list");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].id, "listed-capsule");

        let _ = fs::remove_dir_all(root);
    }

    fn unique_temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "waystone-project-format-{name}-{}",
            std::process::id()
        ))
    }
}
