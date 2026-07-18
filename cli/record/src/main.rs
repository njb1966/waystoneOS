use std::env;
use std::path::Path;
use std::process;
use waystone_audio_metadata::{list_recordings, load_audio_metadata, validate_audio_metadata};
use waystone_cli_output::{escape_json, json_optional_string, print_command_error};

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
            eprintln!("record: usage error");
            print_help();
            2
        }
    }
}

fn list(root: &str, json: bool) -> i32 {
    match list_recordings(Path::new(root)) {
        Ok(recordings) => {
            if json {
                let recordings_json = recordings
                    .iter()
                    .map(|recording| {
                        format!(
                            "{{\"id\":\"{}\",\"title\":\"{}\",\"path\":\"{}\"}}",
                            escape_json(&recording.id),
                            escape_json(&recording.title),
                            escape_json(&recording.path.display().to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"recordings\":[{}]}}}}",
                    recordings_json
                );
            } else if recordings.is_empty() {
                println!("No recordings found");
            } else {
                for recording in recordings {
                    println!(
                        "{}\t{}\t{}",
                        recording.id,
                        recording.title,
                        recording.path.display()
                    );
                }
            }
            0
        }
        Err(error) => print_command_error("record", "list", &error.to_string(), json),
    }
}

fn inspect(path: &str, json: bool) -> i32 {
    match load_audio_metadata(Path::new(path)) {
        Ok(metadata) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"title\":\"{}\",\"master\":\"{}\",\"published\":{}}}}}",
                    escape_json(&metadata.recording.id),
                    escape_json(&metadata.recording.title),
                    escape_json(&metadata.recording.master),
                    json_optional_string(metadata.recording.published.as_deref())
                );
            } else {
                println!("Recording: {}", metadata.recording.title);
                println!("ID: {}", metadata.recording.id);
                println!("Master: {}", metadata.recording.master);
                if let Some(published) = metadata.recording.published {
                    println!("Published: {published}");
                }
            }
            0
        }
        Err(error) => print_command_error("record", "inspect", &error.to_string(), json),
    }
}

fn validate(path: &str, json: bool) -> i32 {
    match validate_audio_metadata(Path::new(path)) {
        Ok(report) => {
            print_validation(
                "recording",
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
        Err(error) => print_command_error("record", "validate", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  record list [--json] ROOT");
    println!("  record inspect [--json] PATH");
    println!("  record validate [--json] PATH");
}

fn print_validation(
    label: &str,
    valid: bool,
    errors: &[waystone_audio_metadata::ValidationIssue],
    warnings: &[waystone_audio_metadata::ValidationIssue],
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

fn json_issues(issues: &[waystone_audio_metadata::ValidationIssue]) -> String {
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
