use std::path::PathBuf;
use waystone_publication_history::{CompletedHistoryOptions, PublicationHistoryRecord};
use waystone_publish_plan::{
    dry_run_publish_with_context, validate_publication_with_context, PublishContext, PublishDryRun,
    PublishPlanError, PublishValidationReport,
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
pub struct ValidatePublicationRequest {
    pub project_path: PathBuf,
    pub target: String,
    pub hosts_root: Option<PathBuf>,
    pub identities_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ValidatePublicationResponse {
    pub report: PublishValidationReport,
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

#[derive(Debug, Clone)]
pub struct BuildCompletedHistoryRequest {
    pub plan: PublishDryRun,
    pub date: String,
    pub transfer_result: String,
    pub verification_result: String,
    pub rollback_available: bool,
    pub rollback_notes: String,
}

#[derive(Debug, Clone)]
pub struct BuildCompletedHistoryResponse {
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

    pub fn validate_publication(
        &self,
        request: ValidatePublicationRequest,
    ) -> Result<ValidatePublicationResponse, PublishPlanError> {
        let report = validate_publication_with_context(
            request.project_path,
            &request.target,
            &PublishContext {
                hosts_root: request.hosts_root,
                identities_root: request.identities_root,
            },
        )?;

        Ok(ValidatePublicationResponse { report })
    }

    pub fn build_planned_history(
        &self,
        request: BuildPlannedHistoryRequest,
    ) -> BuildPlannedHistoryResponse {
        BuildPlannedHistoryResponse {
            record: PublicationHistoryRecord::planned_from_dry_run(&request.plan, request.date),
        }
    }

    pub fn build_completed_history(
        &self,
        request: BuildCompletedHistoryRequest,
    ) -> BuildCompletedHistoryResponse {
        BuildCompletedHistoryResponse {
            record: PublicationHistoryRecord::completed_from_dry_run(
                &request.plan,
                request.date,
                CompletedHistoryOptions {
                    transfer_result: request.transfer_result,
                    verification_result: request.verification_result,
                    rollback_available: request.rollback_available,
                    rollback_notes: request.rollback_notes,
                },
            ),
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

    #[test]
    fn service_builds_completed_history_with_reported_results() {
        let service = PublishService;
        let preview = service
            .preview_publication(PreviewPublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            })
            .expect("preview should succeed");

        let history = service.build_completed_history(BuildCompletedHistoryRequest {
            plan: preview.plan,
            date: "2026-07-20T00:00:00Z".to_string(),
            transfer_result: "completed".to_string(),
            verification_result: "passed".to_string(),
            rollback_available: false,
            rollback_notes: "No rollback snapshot recorded".to_string(),
        });

        assert_eq!(history.record.transfer_result, "completed");
        assert_eq!(history.record.verification_result, "passed");
        assert!(!history.record.rollback.available);
    }

    #[test]
    fn service_validates_publication_readiness() {
        let service = PublishService;
        let validation = service
            .validate_publication(ValidatePublicationRequest {
                project_path: repo_path("examples/projects/ssh-capsule.wayproject"),
                target: "production".to_string(),
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            })
            .expect("validation should succeed");

        assert!(validation.report.valid);
        assert!(!validation.report.blocked);
        assert!(validation.report.errors.is_empty());
    }
}
