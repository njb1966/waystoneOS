use waystone_publish_plan::PublishDryRun;

pub const HISTORY_SCHEMA: u32 = 1;

#[derive(Debug, Clone)]
pub struct PublicationHistoryRecord {
    pub schema: u32,
    pub date: String,
    pub project_id: String,
    pub target: String,
    pub method: String,
    pub destination: Option<String>,
    pub transfer_result: String,
    pub verification_result: String,
    pub files: Vec<PublicationFile>,
    pub rollback: RollbackInfo,
}

#[derive(Debug, Clone)]
pub struct PublicationFile {
    pub path: String,
    pub action: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RollbackInfo {
    pub available: bool,
    pub notes: String,
}

impl PublicationHistoryRecord {
    pub fn planned_from_dry_run(plan: &PublishDryRun, date: impl Into<String>) -> Self {
        let files = plan
            .upload
            .iter()
            .map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-upload".to_string(),
                sha256: None,
            })
            .chain(plan.update.iter().map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-update".to_string(),
                sha256: None,
            }))
            .chain(plan.delete.iter().map(|path| PublicationFile {
                path: path.clone(),
                action: "planned-delete".to_string(),
                sha256: None,
            }))
            .collect();

        Self {
            schema: HISTORY_SCHEMA,
            date: date.into(),
            project_id: plan.project_id.clone(),
            target: plan.target.clone(),
            method: plan.method.clone(),
            destination: plan.destination.clone(),
            transfer_result: "planned".to_string(),
            verification_result: "not-run".to_string(),
            files,
            rollback: RollbackInfo {
                available: false,
                notes: "Dry-run only; no remote state changed".to_string(),
            },
        }
    }

    pub fn to_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("[publication]\n");
        output.push_str(&format!("schema = {}\n", self.schema));
        output.push_str(&format!("date = \"{}\"\n", toml_escape(&self.date)));
        output.push_str(&format!(
            "project_id = \"{}\"\n",
            toml_escape(&self.project_id)
        ));
        output.push_str(&format!("target = \"{}\"\n", toml_escape(&self.target)));
        output.push_str(&format!("method = \"{}\"\n", toml_escape(&self.method)));
        if let Some(destination) = &self.destination {
            output.push_str(&format!("destination = \"{}\"\n", toml_escape(destination)));
        }
        output.push_str(&format!(
            "transfer_result = \"{}\"\n",
            toml_escape(&self.transfer_result)
        ));
        output.push_str(&format!(
            "verification_result = \"{}\"\n",
            toml_escape(&self.verification_result)
        ));
        output.push('\n');

        for file in &self.files {
            output.push_str("[[files]]\n");
            output.push_str(&format!("path = \"{}\"\n", toml_escape(&file.path)));
            output.push_str(&format!("action = \"{}\"\n", toml_escape(&file.action)));
            if let Some(hash) = &file.sha256 {
                output.push_str(&format!("sha256 = \"{}\"\n", toml_escape(hash)));
            }
            output.push('\n');
        }

        output.push_str("[rollback]\n");
        output.push_str(&format!("available = {}\n", self.rollback.available));
        output.push_str(&format!(
            "notes = \"{}\"\n",
            toml_escape(&self.rollback.notes)
        ));

        output
    }
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use waystone_publish_plan::dry_run_publish_with_context;
    use waystone_publish_plan::PublishContext;

    fn repo_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    #[test]
    fn creates_planned_history_record_from_dry_run() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            },
        )
        .expect("dry-run should plan");

        let record = PublicationHistoryRecord::planned_from_dry_run(&plan, "2026-07-17T00:00:00Z");

        assert_eq!(record.schema, HISTORY_SCHEMA);
        assert_eq!(record.transfer_result, "planned");
        assert_eq!(record.verification_result, "not-run");
        assert!(record
            .files
            .iter()
            .any(|file| file.path == "content/index.gmi"));
    }

    #[test]
    fn renders_inspectable_toml() {
        let plan = dry_run_publish_with_context(
            repo_path("examples/projects/ssh-capsule.wayproject"),
            "production",
            &PublishContext {
                hosts_root: Some(repo_path("examples/connections/hosts")),
                identities_root: Some(repo_path("examples/connections/identities")),
            },
        )
        .expect("dry-run should plan");

        let record = PublicationHistoryRecord::planned_from_dry_run(&plan, "2026-07-17T00:00:00Z");
        let toml = record.to_toml();

        assert!(toml.contains("[publication]"));
        assert!(toml.contains("transfer_result = \"planned\""));
        assert!(toml.contains("[[files]]"));
        assert!(toml.contains("[rollback]"));
    }
}
