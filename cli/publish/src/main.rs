use std::env;
use std::path::{Path, PathBuf};
use std::process;
use waystone_cli_output::{
    escape_json, json_optional_string, json_string_array, print_command_error,
};
use waystone_publish_plan::{dry_run_publish_with_context, PublishContext, Resolution};

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
        ["--help"] | ["help"] | [] => {
            print_help();
            0
        }
        _ if positional.contains(&"--dry-run") => dry_run(&positional, json),
        _ => {
            eprintln!("publish: usage error");
            print_help();
            2
        }
    }
}

fn dry_run(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    let context = PublishContext {
        hosts_root: option_value(args, "--hosts").map(PathBuf::from),
        identities_root: option_value(args, "--identities").map(PathBuf::from),
    };

    match dry_run_publish_with_context(Path::new(project), target, &context) {
        Ok(plan) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"destination\":{},\"blocked\":{},\"host_resolution\":{},\"identity_resolution\":{},\"changes\":{{\"upload\":[{}],\"update\":[],\"delete\":[],\"skip\":[]}},\"verification\":{{\"checks\":[{}]}},\"confirmations\":[{}]}}}}",
                    escape_json(&plan.project_id),
                    escape_json(&plan.target),
                    escape_json(&plan.method),
                    json_optional_string(plan.destination.as_deref()),
                    plan.blocked,
                    json_resolution(plan.host_resolution.as_ref()),
                    json_resolution(plan.identity_resolution.as_ref()),
                    json_string_array(&plan.upload),
                    json_string_array(&plan.verification_checks),
                    json_string_array(&plan.confirmations)
                );
            } else {
                println!("Dry-run publication plan");
                println!("Project: {}", plan.project_id);
                println!("Target: {}", plan.target);
                println!("Method: {}", plan.method);
                if let Some(destination) = plan.destination {
                    println!("Destination: {destination}");
                }
                if let Some(host) = plan.host_resolution {
                    println!("Host: {} ({:?}) - {}", host.id, host.status, host.detail);
                }
                if let Some(identity) = plan.identity_resolution {
                    println!(
                        "Identity: {} ({:?}) - {}",
                        identity.id, identity.status, identity.detail
                    );
                }
                if plan.blocked {
                    println!("Blocked: yes");
                }
                println!("Upload:");
                for path in plan.upload {
                    println!("  {path}");
                }
                if !plan.confirmations.is_empty() {
                    println!("Confirmations:");
                    for confirmation in plan.confirmations {
                        println!("  {confirmation}");
                    }
                }
            }
            0
        }
        Err(error) => print_command_error("publish", "dry_run", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  publish --dry-run --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--json]");
}

fn usage_error(message: &str) -> i32 {
    eprintln!("publish: usage error: {message}");
    print_help();
    2
}

fn option_value<'a>(args: &'a [&str], option: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == option)
        .map(|window| window[1])
}

fn json_resolution(resolution: Option<&Resolution>) -> String {
    let Some(resolution) = resolution else {
        return "null".to_string();
    };

    format!(
        "{{\"id\":\"{}\",\"status\":\"{:?}\",\"detail\":\"{}\"}}",
        escape_json(&resolution.id),
        resolution.status,
        escape_json(&resolution.detail)
    )
}
