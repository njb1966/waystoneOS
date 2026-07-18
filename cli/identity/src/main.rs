use std::env;
use std::path::Path;
use std::process;
use waystone_cli_output::{escape_json, print_command_error};
use waystone_host_identity::{list_identities, load_identity, validate_identity};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    process::exit(run(&args));
}

fn run(args: &[String]) -> i32 {
    let json = args.iter().any(|arg| arg == "--json");
    let positional: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| *arg != "--json")
        .collect();

    match positional.as_slice() {
        ["list", root] => list(root, json),
        ["inspect", path] => inspect(path, json),
        ["validate", path] => validate(path, json),
        ["help"] | ["--help"] | [] => {
            print_help();
            0
        }
        _ => {
            eprintln!("identity: usage error");
            print_help();
            2
        }
    }
}

fn list(root: &str, json: bool) -> i32 {
    match list_identities(Path::new(root)) {
        Ok(identities) => {
            if json {
                let identities_json = identities
                    .iter()
                    .map(|identity| {
                        format!(
                            "{{\"id\":\"{}\",\"display_name\":\"{}\",\"path\":\"{}\"}}",
                            escape_json(&identity.id),
                            escape_json(&identity.display_name),
                            escape_json(&identity.path.display().to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"identities\":[{}]}}}}",
                    identities_json
                );
            } else if identities.is_empty() {
                println!("No identities found");
            } else {
                for identity in identities {
                    println!("{}\t{}", identity.id, identity.path.display());
                }
            }
            0
        }
        Err(error) => print_command_error("identity", "list", &error.to_string(), json),
    }
}

fn inspect(path: &str, json: bool) -> i32 {
    match load_identity(Path::new(path)) {
        Ok(record) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"display_name\":\"{}\",\"ssh_keys\":{},\"certificates\":{}}}}}",
                    escape_json(&record.identity.id),
                    escape_json(&record.identity.display_name),
                    record.ssh_keys.len(),
                    record.certificates.len()
                );
            } else {
                println!("Identity: {}", record.identity.display_name);
                println!("ID: {}", record.identity.id);
                for key in record.ssh_keys {
                    println!("SSH key: {}\t{}", key.id, key.public_key);
                }
                for certificate in record.certificates {
                    println!(
                        "Certificate: {}\t{}\t{}",
                        certificate.id, certificate.certificate_type, certificate.fingerprint
                    );
                }
            }
            0
        }
        Err(error) => print_command_error("identity", "inspect", &error.to_string(), json),
    }
}

fn validate(path: &str, json: bool) -> i32 {
    match validate_identity(Path::new(path)) {
        Ok(report) => {
            print_validation(
                "identity",
                report.valid,
                &report.errors,
                &report.warnings,
                json,
            );
            if report.valid {
                0
            } else {
                3
            }
        }
        Err(error) => print_command_error("identity", "validate", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  identity list [--json] ROOT");
    println!("  identity inspect [--json] PATH");
    println!("  identity validate [--json] PATH");
}

fn print_validation(
    label: &str,
    valid: bool,
    errors: &[waystone_host_identity::ValidationIssue],
    warnings: &[waystone_host_identity::ValidationIssue],
    json: bool,
) {
    if json {
        println!(
            "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"valid\":{},\"errors\":[{}],\"warnings\":[{}]}}}}",
            valid,
            json_issues(errors),
            json_issues(warnings)
        );
    } else {
        println!("{label} is {}", if valid { "valid" } else { "invalid" });
        for error in errors {
            println!("error: {}: {}", error.code, error.message);
        }
        for warning in warnings {
            println!("warning: {}: {}", warning.code, warning.message);
        }
    }
}

fn json_issues(issues: &[waystone_host_identity::ValidationIssue]) -> String {
    issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"code\":\"{}\",\"message\":\"{}\"}}",
                issue.code,
                escape_json(&issue.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
