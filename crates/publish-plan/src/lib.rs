use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use waystone_audio_metadata::{
    load_feed_entry_metadata, validate_feed_entry, ValidateFeedEntryOptions,
};
use waystone_host_identity::{list_hosts, list_identities, validate_host, validate_identity};
use waystone_project_format::{load_manifest, validate_project, ProjectFormatError, PublishTarget};

#[derive(Debug, Clone)]
pub struct PublishDryRun {
    pub project_id: String,
    pub target: String,
    pub method: String,
    pub destination: Option<String>,
    pub upload: Vec<String>,
    pub update: Vec<String>,
    pub delete: Vec<String>,
    pub skip: Vec<String>,
    pub confirmations: Vec<String>,
    pub verification_checks: Vec<String>,
    pub host_resolution: Option<Resolution>,
    pub identity_resolution: Option<Resolution>,
    pub feed: FeedPublicationState,
    pub blocked: bool,
}

#[derive(Debug, Clone)]
pub struct FeedPublicationState {
    pub configured: bool,
    pub enabled: bool,
    pub feed_type: Option<String>,
    pub path: Option<String>,
    pub exists: bool,
    pub prepared_entries: usize,
    pub invalid_entries: usize,
    pub invalid_entry_diagnostics: Vec<FeedEntryDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct FeedEntryDiagnostic {
    pub path: String,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub id: String,
    pub status: ResolutionStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionStatus {
    NotRequired,
    Resolved,
    Missing,
    Blocked,
    Invalid,
}

#[derive(Debug, Clone, Default)]
pub struct PublishContext {
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum PublishPlanError {
    #[error("{0}")]
    Project(#[from] ProjectFormatError),

    #[error("project validation failed")]
    ProjectInvalid,

    #[error("publish target not found: {0}")]
    TargetNotFound(String),

    #[error("project files could not be read: {path}: {source}")]
    ReadDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn dry_run_publish(
    project_root: impl AsRef<Path>,
    target_name: &str,
) -> Result<PublishDryRun, PublishPlanError> {
    dry_run_publish_with_context(project_root, target_name, &PublishContext::default())
}

pub fn dry_run_publish_with_context(
    project_root: impl AsRef<Path>,
    target_name: &str,
    context: &PublishContext,
) -> Result<PublishDryRun, PublishPlanError> {
    let project_root = project_root.as_ref();
    let validation = validate_project(project_root)?;
    if !validation.valid {
        return Err(PublishPlanError::ProjectInvalid);
    }

    let manifest = load_manifest(project_root)?;
    let target = find_target(&manifest, target_name)?;
    let project_id = manifest.project.id.clone();
    let target_name = target.name.clone();
    let method = target.method.clone();
    let destination = destination_for(target);
    let host_resolution = resolve_host(target, context);
    let identity_resolution = resolve_identity(target, context);
    let blocked = resolution_blocks(&host_resolution) || resolution_blocks(&identity_resolution);
    let feed = feed_publication_state(project_root, &manifest);
    let mut upload = collect_publishable_files(project_root, &manifest)?;
    upload.sort();

    let confirmations = if target.delete_policy.as_deref() == Some("confirm") {
        vec!["remote deletion requires explicit confirmation".to_string()]
    } else {
        Vec::new()
    };

    Ok(PublishDryRun {
        project_id,
        target: target_name,
        method,
        destination,
        upload,
        update: Vec::new(),
        delete: Vec::new(),
        skip: Vec::new(),
        confirmations,
        verification_checks: vec!["fetch".to_string(), "mime".to_string()],
        host_resolution,
        identity_resolution,
        feed,
        blocked,
    })
}

fn resolution_blocks(resolution: &Option<Resolution>) -> bool {
    resolution.as_ref().is_some_and(|resolution| {
        matches!(
            resolution.status,
            ResolutionStatus::Missing | ResolutionStatus::Blocked | ResolutionStatus::Invalid
        )
    })
}

fn resolve_host(target: &PublishTarget, context: &PublishContext) -> Option<Resolution> {
    let host_id = target.host.as_ref()?;
    let Some(hosts_root) = &context.hosts_root else {
        return Some(Resolution {
            id: host_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "host metadata root was not provided".to_string(),
        });
    };

    let Ok(hosts) = list_hosts(hosts_root) else {
        return Some(Resolution {
            id: host_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "host metadata root could not be read".to_string(),
        });
    };

    let Some(host) = hosts.into_iter().find(|host| host.id == *host_id) else {
        return Some(Resolution {
            id: host_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "host record was not found".to_string(),
        });
    };

    let Ok(validation) = validate_host(&host.path) else {
        return Some(Resolution {
            id: host_id.clone(),
            status: ResolutionStatus::Invalid,
            detail: "host record could not be validated".to_string(),
        });
    };

    if !validation.valid {
        return Some(Resolution {
            id: host_id.clone(),
            status: ResolutionStatus::Invalid,
            detail: "host record is invalid".to_string(),
        });
    }

    Some(Resolution {
        id: host_id.clone(),
        status: ResolutionStatus::Resolved,
        detail: format!("host {} resolved at {}", host.id, host.address),
    })
}

fn resolve_identity(target: &PublishTarget, context: &PublishContext) -> Option<Resolution> {
    let identity_id = target.identity.as_ref()?;
    let Some(identities_root) = &context.identities_root else {
        return Some(Resolution {
            id: identity_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "identity metadata root was not provided".to_string(),
        });
    };

    let Ok(identities) = list_identities(identities_root) else {
        return Some(Resolution {
            id: identity_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "identity metadata root could not be read".to_string(),
        });
    };

    let Some(identity) = identities
        .into_iter()
        .find(|identity| identity.id == *identity_id)
    else {
        return Some(Resolution {
            id: identity_id.clone(),
            status: ResolutionStatus::Missing,
            detail: "identity record was not found".to_string(),
        });
    };

    let Ok(validation) = validate_identity(&identity.path) else {
        return Some(Resolution {
            id: identity_id.clone(),
            status: ResolutionStatus::Invalid,
            detail: "identity record could not be validated".to_string(),
        });
    };

    if !validation.valid {
        return Some(Resolution {
            id: identity_id.clone(),
            status: ResolutionStatus::Invalid,
            detail: "identity record is invalid".to_string(),
        });
    }

    Some(Resolution {
        id: identity_id.clone(),
        status: ResolutionStatus::Resolved,
        detail: format!("identity {} resolved", identity.id),
    })
}

fn find_target<'a>(
    manifest: &'a waystone_project_format::Manifest,
    target_name: &str,
) -> Result<&'a PublishTarget, PublishPlanError> {
    manifest
        .publish
        .as_ref()
        .and_then(|publish| publish.targets.as_ref())
        .and_then(|targets| targets.iter().find(|target| target.name == target_name))
        .ok_or_else(|| PublishPlanError::TargetNotFound(target_name.to_string()))
}

fn destination_for(target: &PublishTarget) -> Option<String> {
    target
        .url
        .clone()
        .or_else(|| target.host.clone())
        .or_else(|| target.path.clone())
}

fn collect_publishable_files(
    project_root: &Path,
    manifest: &waystone_project_format::Manifest,
) -> Result<Vec<String>, PublishPlanError> {
    let mut files = Vec::new();
    collect_relative_files(project_root, Path::new(&manifest.content.root), &mut files)?;

    if let Some(feed) = &manifest.feed {
        if let Some(path) = &feed.path {
            let feed_path = project_root.join(path);
            if feed_path.is_file() {
                files.push(path.clone());
            }
        }
    }

    if let Some(audio) = &manifest.audio {
        if let Some(published) = &audio.published {
            collect_relative_files(project_root, Path::new(published), &mut files)?;
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn feed_publication_state(
    project_root: &Path,
    manifest: &waystone_project_format::Manifest,
) -> FeedPublicationState {
    let Some(feed) = &manifest.feed else {
        return FeedPublicationState {
            configured: false,
            enabled: false,
            feed_type: None,
            path: None,
            exists: false,
            prepared_entries: 0,
            invalid_entries: 0,
            invalid_entry_diagnostics: Vec::new(),
        };
    };

    let path = feed.path.clone();
    let exists = path
        .as_ref()
        .is_some_and(|path| project_root.join(path).is_file());
    let (prepared_entries, invalid_entries, invalid_entry_diagnostics) =
        prepared_feed_entry_counts(project_root, path.as_deref());

    FeedPublicationState {
        configured: true,
        enabled: feed.enabled.unwrap_or(false),
        feed_type: feed.feed_type.clone(),
        path,
        exists,
        prepared_entries,
        invalid_entries,
        invalid_entry_diagnostics,
    }
}

fn prepared_feed_entry_counts(
    project_root: &Path,
    feed_path: Option<&str>,
) -> (usize, usize, Vec<FeedEntryDiagnostic>) {
    let entries_root = project_root.join("feeds/entries");
    if !entries_root.is_dir() {
        return (0, 0, Vec::new());
    }

    let Ok(entries) = fs::read_dir(&entries_root) else {
        return (
            0,
            1,
            vec![FeedEntryDiagnostic {
                path: "feeds/entries".to_string(),
                issues: vec!["feed entries directory could not be read".to_string()],
            }],
        );
    };

    let mut prepared = 0;
    let mut invalid = 0;
    let mut diagnostics = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            invalid += 1;
            diagnostics.push(FeedEntryDiagnostic {
                path: "feeds/entries".to_string(),
                issues: vec!["feed entry directory item could not be read".to_string()],
            });
            continue;
        };
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        let Ok(report) = validate_feed_entry(&ValidateFeedEntryOptions {
            project_root: project_root.to_path_buf(),
            feed_entry_path: path.clone(),
        }) else {
            invalid += 1;
            diagnostics.push(FeedEntryDiagnostic {
                path: relative_project_path(project_root, &path),
                issues: vec!["feed entry could not be validated".to_string()],
            });
            continue;
        };
        if !report.valid {
            invalid += 1;
            diagnostics.push(FeedEntryDiagnostic {
                path: relative_project_path(project_root, &path),
                issues: report
                    .errors
                    .iter()
                    .map(|issue| issue.message.clone())
                    .collect(),
            });
            continue;
        }

        let Ok(metadata) = load_feed_entry_metadata(&path) else {
            invalid += 1;
            diagnostics.push(FeedEntryDiagnostic {
                path: relative_project_path(project_root, &path),
                issues: vec!["feed entry metadata could not be loaded".to_string()],
            });
            continue;
        };
        let entry_feed = metadata.entry.and_then(|entry| entry.feed);
        if feed_path.is_none() || entry_feed.as_deref() == feed_path {
            prepared += 1;
        }
    }

    (prepared, invalid, diagnostics)
}

fn relative_project_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn collect_relative_files(
    project_root: &Path,
    relative_root: &Path,
    files: &mut Vec<String>,
) -> Result<(), PublishPlanError> {
    let root = project_root.join(relative_root);
    if !root.exists() {
        return Ok(());
    }

    collect_relative_files_inner(project_root, &root, files)
}

fn collect_relative_files_inner(
    project_root: &Path,
    current: &Path,
    files: &mut Vec<String>,
) -> Result<(), PublishPlanError> {
    let entries =
        fs::read_dir(current).map_err(|source| PublishPlanError::ReadDirectoryFailed {
            path: current.to_path_buf(),
            source,
        })?;

    for entry in entries {
        let entry = entry.map_err(|source| PublishPlanError::ReadDirectoryFailed {
            path: current.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_inner(project_root, &path, files)?;
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(project_root) {
                files.push(relative.to_string_lossy().to_string());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    fn temp_project_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "waystone-publish-plan-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        ))
    }

    #[test]
    fn plans_audio_capsule_export() {
        let plan = dry_run_publish(
            repo_path("examples/projects/audio-capsule.wayproject"),
            "export",
        )
        .expect("audio capsule dry-run should plan");

        assert_eq!(plan.project_id, "audio-capsule");
        assert_eq!(plan.target, "export");
        assert_eq!(plan.method, "removable");
        assert!(plan.upload.iter().any(|path| path == "content/index.gmi"));
        assert!(plan.upload.iter().any(|path| path == "feeds/feed.xml"));
        assert!(plan
            .upload
            .iter()
            .any(|path| path == "audio/published/field-note.opus"));
        assert!(plan.feed.configured);
        assert!(plan.feed.enabled);
        assert_eq!(plan.feed.path.as_deref(), Some("feeds/feed.xml"));
        assert!(plan.feed.exists);
        assert_eq!(plan.feed.prepared_entries, 0);
        assert_eq!(plan.feed.invalid_entries, 0);
        assert!(plan.feed.invalid_entry_diagnostics.is_empty());
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn reports_prepared_feed_entry_state() {
        let project = temp_project_root("feed-state").join("feed-state.wayproject");
        fs::create_dir_all(project.join("content")).expect("content directory");
        fs::create_dir_all(project.join("audio/metadata")).expect("metadata directory");
        fs::create_dir_all(project.join("audio/masters")).expect("masters directory");
        fs::create_dir_all(project.join("audio/published")).expect("published directory");
        fs::create_dir_all(project.join("feeds/entries")).expect("feed entries directory");
        fs::write(project.join("content/index.gmi"), "# Feed State\n").expect("content");
        fs::write(project.join("audio/masters/note.flac"), b"master").expect("master");
        fs::write(project.join("audio/published/note.opus"), b"published").expect("published");
        fs::write(
            project.join("project.toml"),
            r#"[waystone]
schema = 1
created_by = "WaystoneOS"

[project]
id = "feed-state"
name = "Feed State"
type = "audio-series"

[content]
root = "content"
index = "index.gmi"

[audio]
masters = "audio/masters"
published = "audio/published"
metadata = "audio/metadata"

[feed]
enabled = true
type = "atom"
path = "feeds/feed.xml"
title = "Feed State"

[[publish.targets]]
name = "export"
method = "removable"
path = "publish/export"
delete_policy = "forbid"
"#,
        )
        .expect("manifest");
        fs::write(
            project.join("audio/metadata/note.toml"),
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
        fs::write(
            project.join("feeds/feed.xml"),
            r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Feed State</title>
</feed>
"#,
        )
        .expect("feed xml");
        fs::write(
            project.join("feeds/entries/note.toml"),
            r#"[entry]
id = "tag:example.invalid,2026:note"
title = "Note"
updated = "2026-07-19T00:00:00Z"
summary = "Note summary"
feed = "feeds/feed.xml"
recording = "note"
recording_metadata = "audio/metadata/note.toml"

[enclosure]
path = "audio/published/note.opus"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("feed entry");
        fs::write(
            project.join("feeds/entries/broken.toml"),
            r#"[entry]
id = "tag:example.invalid,2026:broken"
title = "Broken"
updated = "2026-07-19T00:00:00Z"
summary = "Broken summary"
feed = "feeds/feed.xml"
recording = "broken"

[enclosure]
path = "audio/published/missing.opus"
mime_type = "audio/ogg; codecs=opus"
"#,
        )
        .expect("broken feed entry");

        let plan = dry_run_publish(&project, "export").expect("dry-run should plan");

        assert!(plan.feed.configured);
        assert_eq!(plan.feed.feed_type.as_deref(), Some("atom"));
        assert!(plan.feed.exists);
        assert_eq!(plan.feed.prepared_entries, 1);
        assert_eq!(plan.feed.invalid_entries, 1);
        assert_eq!(plan.feed.invalid_entry_diagnostics.len(), 1);
        assert_eq!(
            plan.feed.invalid_entry_diagnostics[0].path,
            "feeds/entries/broken.toml"
        );
        assert!(plan.feed.invalid_entry_diagnostics[0]
            .issues
            .iter()
            .any(|issue| issue.contains("entry.recording_metadata")));

        let _ = fs::remove_dir_all(project.parent().expect("temp project has parent"));
    }

    #[test]
    fn refuses_missing_target() {
        let error = dry_run_publish(
            repo_path("examples/projects/audio-capsule.wayproject"),
            "production",
        )
        .expect_err("missing target should fail");

        assert!(matches!(error, PublishPlanError::TargetNotFound(_)));
    }

    #[test]
    fn resolves_host_and_identity_for_ssh_target() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            },
        )
        .expect("ssh capsule dry-run should plan");

        assert!(!plan.blocked);
        assert_eq!(
            plan.host_resolution.as_ref().map(|value| &value.status),
            Some(&ResolutionStatus::Resolved)
        );
        assert_eq!(
            plan.identity_resolution.as_ref().map(|value| &value.status),
            Some(&ResolutionStatus::Resolved)
        );
    }

    #[test]
    fn blocks_when_host_metadata_is_missing() {
        let plan = dry_run_publish(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
        )
        .expect("ssh capsule dry-run should still produce local plan");

        assert!(plan.blocked);
        assert_eq!(
            plan.host_resolution.as_ref().map(|value| &value.status),
            Some(&ResolutionStatus::Missing)
        );
    }
}
