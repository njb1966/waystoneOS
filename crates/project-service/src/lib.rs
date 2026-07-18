use std::path::PathBuf;
use waystone_project_format::{
    create_project, inspect_project, list_projects, validate_project, ProjectCreateOptions,
    ProjectFormatError, ProjectInspection, ProjectSummary, ValidationReport,
};

#[derive(Debug, Default)]
pub struct ProjectService;

#[derive(Debug, Clone)]
pub struct CreateProjectRequest {
    pub parent: PathBuf,
    pub id: String,
    pub name: String,
    pub project_type: String,
    pub content_index: String,
    pub language: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateProjectResponse {
    pub project_path: PathBuf,
    pub schema: u32,
}

#[derive(Debug, Clone)]
pub struct ListProjectsRequest {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InspectProjectRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateProjectRequest {
    pub path: PathBuf,
}

impl ProjectService {
    pub fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> Result<CreateProjectResponse, ProjectFormatError> {
        let created = create_project(&ProjectCreateOptions {
            parent: request.parent,
            id: request.id,
            name: request.name,
            project_type: request.project_type,
            content_index: request.content_index,
            language: request.language,
            author: request.author,
        })?;

        Ok(CreateProjectResponse {
            project_path: created.project_path,
            schema: created.schema,
        })
    }

    pub fn list_projects(
        &self,
        request: ListProjectsRequest,
    ) -> Result<Vec<ProjectSummary>, ProjectFormatError> {
        list_projects(request.root)
    }

    pub fn inspect_project(
        &self,
        request: InspectProjectRequest,
    ) -> Result<ProjectInspection, ProjectFormatError> {
        inspect_project(request.path)
    }

    pub fn validate_project(
        &self,
        request: ValidateProjectRequest,
    ) -> Result<ValidationReport, ProjectFormatError> {
        validate_project(request.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn service_creates_and_lists_project() {
        let root =
            std::env::temp_dir().join(format!("waystone-project-service-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temp root should be created");

        let service = ProjectService;
        let created = service
            .create_project(CreateProjectRequest {
                parent: root.clone(),
                id: "service-capsule".to_string(),
                name: "Service Capsule".to_string(),
                project_type: "capsule".to_string(),
                content_index: "index.gmi".to_string(),
                language: Some("en".to_string()),
                author: None,
            })
            .expect("project should be created");

        let report = service
            .validate_project(ValidateProjectRequest {
                path: created.project_path,
            })
            .expect("created project should validate");
        assert!(report.valid, "{report:#?}");

        let projects = service
            .list_projects(ListProjectsRequest { root: root.clone() })
            .expect("projects should list");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].id, "service-capsule");

        let _ = fs::remove_dir_all(root);
    }
}
