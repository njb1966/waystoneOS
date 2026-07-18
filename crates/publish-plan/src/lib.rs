use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
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
    pub blocked: bool,
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

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
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
        assert!(plan.delete.is_empty());
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
