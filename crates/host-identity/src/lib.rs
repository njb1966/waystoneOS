use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct HostRecord {
    pub host: HostSection,
    #[serde(default)]
    pub services: Vec<HostService>,
}

#[derive(Debug, Deserialize)]
pub struct HostSection {
    pub id: String,
    pub display_name: String,
    pub address: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HostService {
    #[serde(rename = "type")]
    pub service_type: String,
    pub port: u16,
    pub trust: String,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdentityRecord {
    pub identity: IdentitySection,
    #[serde(default)]
    pub ssh_keys: Vec<SshKeyRecord>,
    #[serde(default)]
    pub certificates: Vec<CertificateRecord>,
}

#[derive(Debug, Deserialize)]
pub struct IdentitySection {
    pub id: String,
    pub display_name: String,
    pub author_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SshKeyRecord {
    pub id: String,
    pub public_key: String,
    pub private_key_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct CertificateRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub certificate_type: String,
    pub fingerprint: String,
    pub private_key_ref: String,
    pub expires: Option<String>,
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
pub struct HostSummary {
    pub id: String,
    pub display_name: String,
    pub address: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct IdentitySummary {
    pub id: String,
    pub display_name: String,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum HostIdentityError {
    #[error("record path does not exist: {0}")]
    NotFound(PathBuf),

    #[error("record path is not a file: {0}")]
    NotFile(PathBuf),

    #[error("record could not be read: {path}: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("record could not be parsed: {0}")]
    ParseFailed(toml::de::Error),

    #[error("directory path does not exist: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("directory path is not a directory: {0}")]
    NotDirectory(PathBuf),

    #[error("directory could not be read: {path}: {source}")]
    ReadDirectoryFailed {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn load_host(path: impl AsRef<Path>) -> Result<HostRecord, HostIdentityError> {
    let text = read_record(path.as_ref())?;
    toml::from_str(&text).map_err(HostIdentityError::ParseFailed)
}

pub fn load_identity(path: impl AsRef<Path>) -> Result<IdentityRecord, HostIdentityError> {
    let text = read_record(path.as_ref())?;
    toml::from_str(&text).map_err(HostIdentityError::ParseFailed)
}

pub fn validate_host(path: impl AsRef<Path>) -> Result<ValidationReport, HostIdentityError> {
    let host = load_host(path)?;
    Ok(validate_host_record(&host))
}

pub fn validate_identity(path: impl AsRef<Path>) -> Result<ValidationReport, HostIdentityError> {
    let text = read_record(path.as_ref())?;
    let identity: IdentityRecord = toml::from_str(&text).map_err(HostIdentityError::ParseFailed)?;
    let mut report = validate_identity_record(&identity);

    if contains_private_key_material(&text) {
        report.errors.push(error(
            "private_key_material_present",
            "identity record appears to contain private key material".to_string(),
            Some(path.as_ref().display().to_string()),
        ));
        report.valid = false;
    }

    Ok(report)
}

pub fn list_hosts(root: impl AsRef<Path>) -> Result<Vec<HostSummary>, HostIdentityError> {
    let mut hosts = Vec::new();
    for path in list_toml_files(root.as_ref())? {
        if let Ok(host) = load_host(&path) {
            hosts.push(HostSummary {
                id: host.host.id,
                display_name: host.host.display_name,
                address: host.host.address,
                path,
            });
        }
    }
    hosts.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(hosts)
}

pub fn list_identities(root: impl AsRef<Path>) -> Result<Vec<IdentitySummary>, HostIdentityError> {
    let mut identities = Vec::new();
    for path in list_toml_files(root.as_ref())? {
        if let Ok(identity) = load_identity(&path) {
            identities.push(IdentitySummary {
                id: identity.identity.id,
                display_name: identity.identity.display_name,
                path,
            });
        }
    }
    identities.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(identities)
}

pub fn validate_host_record(host: &HostRecord) -> ValidationReport {
    let mut issues = Vec::new();
    validate_id(&mut issues, "host.id", &host.host.id);

    if host.host.address.trim().is_empty() {
        issues.push(error(
            "missing_host_address",
            "host address is required".to_string(),
            Some("host.address"),
        ));
    }

    for service in &host.services {
        if service.service_type.trim().is_empty() {
            issues.push(error(
                "missing_service_type",
                "service type is required".to_string(),
                Some("services.type"),
            ));
        }
        if !supported_trust_state(&service.trust) {
            issues.push(error(
                "unsupported_trust_state",
                format!("unsupported trust state: {}", service.trust),
                Some("services.trust"),
            ));
        }
    }

    ValidationReport::from_issues(issues)
}

pub fn validate_identity_record(identity: &IdentityRecord) -> ValidationReport {
    let mut issues = Vec::new();
    validate_id(&mut issues, "identity.id", &identity.identity.id);

    for key in &identity.ssh_keys {
        validate_id(&mut issues, "ssh_keys.id", &key.id);
        if !key.public_key.starts_with("ssh-") {
            issues.push(error(
                "invalid_public_key",
                format!("SSH key {} does not look like a public key", key.id),
                Some("ssh_keys.public_key"),
            ));
        }
        validate_secret_ref(
            &mut issues,
            "ssh_keys.private_key_ref",
            &key.private_key_ref,
        );
    }

    for certificate in &identity.certificates {
        validate_id(&mut issues, "certificates.id", &certificate.id);
        if !certificate.fingerprint.starts_with("sha256:") {
            issues.push(warning(
                "unusual_certificate_fingerprint",
                format!(
                    "certificate {} fingerprint does not start with sha256:",
                    certificate.id
                ),
                Some("certificates.fingerprint"),
            ));
        }
        validate_secret_ref(
            &mut issues,
            "certificates.private_key_ref",
            &certificate.private_key_ref,
        );
    }

    ValidationReport::from_issues(issues)
}

fn read_record(path: &Path) -> Result<String, HostIdentityError> {
    if !path.exists() {
        return Err(HostIdentityError::NotFound(path.to_path_buf()));
    }
    if !path.is_file() {
        return Err(HostIdentityError::NotFile(path.to_path_buf()));
    }
    fs::read_to_string(path).map_err(|source| HostIdentityError::Unreadable {
        path: path.to_path_buf(),
        source,
    })
}

fn list_toml_files(root: &Path) -> Result<Vec<PathBuf>, HostIdentityError> {
    if !root.exists() {
        return Err(HostIdentityError::DirectoryNotFound(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(HostIdentityError::NotDirectory(root.to_path_buf()));
    }

    let entries = fs::read_dir(root).map_err(|source| HostIdentityError::ReadDirectoryFailed {
        path: root.to_path_buf(),
        source,
    })?;

    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| HostIdentityError::ReadDirectoryFailed {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("toml") {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn validate_id(issues: &mut Vec<ValidationIssue>, field: &'static str, value: &str) {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));

    if !valid {
        issues.push(error(
            "invalid_id",
            format!("{field} must contain only ASCII letters, digits, '-' or '_'"),
            Some(field),
        ));
    }
}

fn validate_secret_ref(issues: &mut Vec<ValidationIssue>, field: &'static str, value: &str) {
    if !value.starts_with("workspace-secret:") {
        issues.push(error(
            "invalid_secret_reference",
            format!("{field} must use workspace-secret: reference"),
            Some(field),
        ));
    }
}

fn supported_trust_state(value: &str) -> bool {
    matches!(
        value,
        "unknown" | "observed" | "trusted" | "changed" | "blocked"
    )
}

fn contains_private_key_material(text: &str) -> bool {
    text.contains("BEGIN OPENSSH PRIVATE KEY")
        || text.contains("BEGIN RSA PRIVATE KEY")
        || text.contains("BEGIN EC PRIVATE KEY")
        || text.contains("BEGIN PRIVATE KEY")
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
    fn validates_host_example() {
        let report = validate_host(repo_path("examples/connections/hosts/offgridholdout.toml"))
            .expect("host example should load");

        assert!(report.valid, "{report:#?}");
    }

    #[test]
    fn validates_identity_example() {
        let report = validate_identity(repo_path("examples/connections/identities/nick-pub.toml"))
            .expect("identity example should load");

        assert!(report.valid, "{report:#?}");
    }

    #[test]
    fn rejects_invalid_trust_state() {
        let report = validate_host(repo_path("tests/fixtures/hosts/invalid-trust/host.toml"))
            .expect("invalid host should parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "unsupported_trust_state"));
    }

    #[test]
    fn detects_private_key_material() {
        let report = validate_identity(repo_path(
            "tests/fixtures/identities/private-key-leak/identity.toml",
        ))
        .expect("invalid identity should parse");

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "private_key_material_present"));
    }

    #[test]
    fn lists_hosts() {
        let hosts = list_hosts(repo_path("examples/connections/hosts")).expect("hosts should list");

        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].id, "offgridholdout");
    }

    #[test]
    fn lists_identities() {
        let identities =
            list_identities(repo_path("examples/connections/identities")).expect("ids should list");

        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].id, "nick-pub");
    }
}
