use std::path::PathBuf;
use waystone_publication_history::PublicationHistoryRecord;
use waystone_publish_plan::{
    dry_run_publish_with_context, PublishContext, PublishDryRun, PublishPlanError,
};

#[derive(Debug, Default)]
pub struct PublishService;

#[derive(Debug, Clone)]
pub struct PreviewPublicationRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PreviewPublicationResponse {
    pub plan: PublishDryRun,
}

#[derive(Debug, Clone)]
pub struct BuildPlannedHistoryRequest {
    pub plan: PublishDryRun,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct BuildPlannedHistoryResponse {
    pub record: PublicationHistoryRecord,
}

impl PublishService {
    pub fn preview_publication(
        &self,
        request: PreviewPublicationRequest,
    ) -> Result<PreviewPublicationResponse, PublishPlanError> {
        let plan = dry_run_publish_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
            },
        )?;

        Ok(PreviewPublicationResponse { plan })
    }

    pub fn build_planned_history(
        &self,
        request: BuildPlannedHistoryRequest,
    ) -> BuildPlannedHistoryResponse {
        BuildPlannedHistoryResponse {
            record: PublicationHistoryRecord::planned_from_dry_run(&request.plan, request.date),
        }
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
    fn service_previews_and_builds_planned_history() {
        let service = PublishService;
        let preview = service
            .preview_publication(PreviewPublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            })
            .expect("preview should succeed");

        assert!(!preview.plan.blocked);

        let history = service.build_planned_history(BuildPlannedHistoryRequest {
            plan: preview.plan,
            date: "2026-07-17T00:00:00Z".to_string(),
        });

        assert_eq!(history.record.transfer_result, "planned");
    }
}
