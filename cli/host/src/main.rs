use std::env;
use std::path::Path;
use std::process;
use waystone_cli_output::{escape_json, print_command_error};
use waystone_host_identity::{list_hosts, load_host, validate_host};

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
            eprintln!("host: usage error");
            print_help();
            2
        }
    }
}

fn list(root: &str, json: bool) -> i32 {
    match list_hosts(Path::new(root)) {
        Ok(hosts) => {
            if json {
                let hosts_json = hosts
                    .iter()
                    .map(|host| {
                        format!(
                            "{{\"id\":\"{}\",\"display_name\":\"{}\",\"address\":\"{}\",\"path\":\"{}\"}}",
                            escape_json(&host.id),
                            escape_json(&host.display_name),
                            escape_json(&host.address),
                            escape_json(&host.path.display().to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"hosts\":[{}]}}}}",
                    hosts_json
                );
            } else if hosts.is_empty() {
                println!("No hosts found");
            } else {
                for host in hosts {
                    println!("{}\t{}\t{}", host.id, host.address, host.path.display());
                }
            }
            0
        }
        Err(error) => print_command_error("host", "list", &error.to_string(), json),
    }
}

fn inspect(path: &str, json: bool) -> i32 {
    match load_host(Path::new(path)) {
        Ok(record) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"display_name\":\"{}\",\"address\":\"{}\",\"services\":{}}}}}",
                    escape_json(&record.host.id),
                    escape_json(&record.host.display_name),
                    escape_json(&record.host.address),
                    record.services.len()
                );
            } else {
                println!("Host: {}", record.host.display_name);
                println!("ID: {}", record.host.id);
                println!("Address: {}", record.host.address);
                for service in record.services {
                    println!(
                        "Service: {}\tport {}\ttrust {}",
                        service.service_type, service.port, service.trust
                    );
                }
            }
            0
        }
        Err(error) => print_command_error("host", "inspect", &error.to_string(), json),
    }
}

fn validate(path: &str, json: bool) -> i32 {
    match validate_host(Path::new(path)) {
        Ok(report) => {
            print_validation("host", report.valid, &report.errors, &report.warnings, json);
            if report.valid {
                0
            } else {
                3
            }
        }
        Err(error) => print_command_error("host", "validate", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  host list [--json] ROOT");
    println!("  host inspect [--json] PATH");
    println!("  host validate [--json] PATH");
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
