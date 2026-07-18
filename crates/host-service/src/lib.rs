use std::path::PathBuf;
use waystone_host_identity::{
    list_hosts, load_host, validate_host, HostIdentityError, HostRecord, HostSummary,
    ValidationReport,
};

#[derive(Debug, Default)]
pub struct HostService;

#[derive(Debug, Clone)]
pub struct ListHostsRequest {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InspectHostRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidateHostRequest {
    pub path: PathBuf,
}

impl HostService {
    pub fn list_hosts(
        &self,
        request: ListHostsRequest,
    ) -> Result<Vec<HostSummary>, HostIdentityError> {
        list_hosts(request.root)
    }

    pub fn inspect_host(
        &self,
        request: InspectHostRequest,
    ) -> Result<HostRecord, HostIdentityError> {
        load_host(request.path)
    }

    pub fn validate_host(
        &self,
        request: ValidateHostRequest,
    ) -> Result<ValidationReport, HostIdentityError> {
        validate_host(request.path)
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
    fn service_lists_and_validates_hosts() {
        let service = HostService;
        let hosts = service
            .list_hosts(ListHostsRequest {
                root: repo_path("examples/connections/hosts"),
            })
            .expect("hosts should list");

        assert_eq!(hosts.len(), 1);

        let report = service
            .validate_host(ValidateHostRequest {
                path: hosts[0].path.clone(),
            })
            .expect("host should validate");

        assert!(report.valid, "{report:#?}");
    }
}
