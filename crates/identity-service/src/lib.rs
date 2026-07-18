use std::path::PathBuf;
use waystone_host_identity::{
    list_identities, load_identity, validate_identity, HostIdentityError, IdentityRecord,
    IdentitySummary, ValidationReport,
};

#[derive(Debug, Default)]
pub struct IdentityService;

#[derive(Debug, Clone)]
pub struct ListIdentitiesRequest {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InspectIdentityRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateIdentityRequest {
    pub path: PathBuf,
}

impl IdentityService {
    pub fn list_identities(
        &self,
        request: ListIdentitiesRequest,
    ) -> Result<Vec<IdentitySummary>, HostIdentityError> {
        list_identities(request.root)
    }

    pub fn inspect_identity(
        &self,
        request: InspectIdentityRequest,
    ) -> Result<IdentityRecord, HostIdentityError> {
        load_identity(request.path)
    }

    pub fn validate_identity(
        &self,
        request: ValidateIdentityRequest,
    ) -> Result<ValidationReport, HostIdentityError> {
        validate_identity(request.path)
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
    fn service_lists_and_validates_identities() {
        let service = IdentityService;
        let identities = service
            .list_identities(ListIdentitiesRequest {
                root: repo_path("examples/connections/identities"),
            })
            .expect("identities should list");

        assert_eq!(identities.len(), 1);

        let report = service
            .validate_identity(ValidateIdentityRequest {
                path: identities[0].path.clone(),
            })
            .expect("identity should validate");

        assert!(report.valid, "{report:#?}");
    }
}
