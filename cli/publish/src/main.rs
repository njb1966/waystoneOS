use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use waystone_cli_output::{
    escape_json, json_optional_string, json_string_array, print_command_error,
};
use waystone_publication_history::{
    list_completed_history_records, list_planned_history_previews, read_completed_history_record,
    read_planned_history_preview, write_completed_history_record, write_planned_history_preview,
    CompletedHistoryDetail, CompletedHistoryEntry, CompletedHistoryOptions,
    PlannedHistoryPreviewDetail, PlannedHistoryPreviewEntry, PublicationHistoryRecord,
};
use waystone_publish_plan::{
    dry_run_publish_with_context, export_remote_state_manifest, inspect_remote_state_manifest,
    prepare_removable_execution_with_context, remote_state_manifest_text,
    transfer_intent_with_context, validate_publication_with_context, FeedEntryDiagnostic,
    FeedPublicationState, PublishContext, PublishValidationIssue, RemoteComparisonState,
    RemoteStateManifest, RemovableExecutionOperation, RemovableExecutionPlan, Resolution,
    TransferIntent,
};
use waystone_publish_service::{
    ExecuteRemovableRequest, ExecuteRemovableResponse, PublishService, RemovableExecutionFileResult,
};

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
        _ if positional.contains(&"--export-remote-state") => {
            export_remote_state(&positional, json)
        }
        _ if positional.contains(&"--inspect-remote-state") => {
            inspect_remote_state(&positional, json)
        }
        _ if positional.contains(&"--execute-removable") => execute_removable(&positional, json),
        _ if positional.contains(&"--prepare-removable-execution") => {
            prepare_removable_execution(&positional, json)
        }
        _ if positional.contains(&"--transfer-intent") => transfer_intent(&positional, json),
        _ if positional.contains(&"--validate") => validate_publication_command(&positional, json),
        _ if positional.contains(&"--list-completed-history") => {
            list_completed_history_files(&positional, json)
        }
        _ if positional.contains(&"--read-completed-history") => {
            read_completed_history_file(&positional, json)
        }
        _ if positional.contains(&"--save-completed-history") => {
            save_completed_history(&positional, json)
        }
        _ if positional.contains(&"--completed-history") => completed_history(&positional, json),
        _ if positional.contains(&"--list-planned-history-previews") => {
            list_planned_history_preview_files(&positional, json)
        }
        _ if positional.contains(&"--read-planned-history-preview") => {
            read_planned_history_preview_file(&positional, json)
        }
        _ if positional.contains(&"--save-planned-history-preview") => {
            save_planned_history_preview(&positional, json)
        }
        _ if positional.contains(&"--planned-history") => planned_history(&positional, json),
        _ if positional.contains(&"--dry-run") => dry_run(&positional, json),
        _ => {
            eprintln!("publish: usage error");
            print_help();
            2
        }
    }
}

fn export_remote_state(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    match export_remote_state_manifest(Path::new(project), target) {
        Ok(manifest) => {
            let manifest_text = remote_state_manifest_text(&manifest.paths);
            let output_path = option_value(args, "--output");
            if let Some(output_path) = output_path {
                if let Err(error) = write_new_file(Path::new(output_path), manifest_text.as_bytes())
                {
                    return print_command_error("publish", "export_remote_state", &error, json);
                }
            }

            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{{},\"path_count\":{},\"paths\":[{}],\"manifest\":\"{}\",\"output_path\":{}}}}}",
                    json_remote_state_manifest_identity(&manifest),
                    manifest.paths.len(),
                    json_string_array(&manifest.paths),
                    escape_json(&manifest_text),
                    json_optional_string(output_path)
                );
            } else if let Some(output_path) = output_path {
                println!(
                    "Saved remote-state manifest: {} ({} paths)",
                    output_path,
                    manifest.paths.len()
                );
            } else {
                print!("{manifest_text}");
            }
            0
        }
        Err(error) => {
            print_command_error("publish", "export_remote_state", &error.to_string(), json)
        }
    }
}

fn inspect_remote_state(args: &[&str], json: bool) -> i32 {
    let Some(remote_state) = option_value(args, "--remote-state") else {
        return usage_error("missing --remote-state");
    };

    match inspect_remote_state_manifest(Path::new(remote_state)) {
        Ok(manifest) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{{},\"path_count\":{},\"paths\":[{}]}}}}",
                    json_remote_state_manifest_identity(&manifest),
                    manifest.paths.len(),
                    json_string_array(&manifest.paths)
                );
            } else {
                println!("Remote-state manifest: {remote_state}");
                println!("Paths: {}", manifest.paths.len());
                for path in manifest.paths {
                    println!("  {path}");
                }
            }
            0
        }
        Err(error) => {
            print_command_error("publish", "inspect_remote_state", &error.to_string(), json)
        }
    }
}

fn validate_publication_command(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    let context = publish_context(args);
    match validate_publication_with_context(Path::new(project), target, &context) {
        Ok(report) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"valid\":{},\"blocked\":{},\"errors\":[{}],\"warnings\":[{}]}}}}",
                    escape_json(&report.project_id),
                    escape_json(&report.target),
                    report.valid,
                    report.blocked,
                    json_publish_validation_issues(&report.errors),
                    json_publish_validation_issues(&report.warnings)
                );
            } else if report.valid {
                println!("Publication target is valid");
                for warning in report.warnings {
                    println!("warning: {}: {}", warning.code, warning.message);
                }
            } else {
                println!("Publication target is invalid");
                for error in report.errors {
                    println!("error: {}: {}", error.code, error.message);
                }
                for warning in report.warnings {
                    println!("warning: {}: {}", warning.code, warning.message);
                }
            }
            0
        }
        Err(error) => {
            print_command_error("publish", "validate_publication", &error.to_string(), json)
        }
    }
}

fn transfer_intent(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    let context = publish_context(args);
    match transfer_intent_with_context(Path::new(project), target, &context) {
        Ok(intent) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{}}}",
                    json_transfer_intent(&intent)
                );
            } else {
                println!("Publication transfer intent");
                println!("Project: {}", intent.project_id);
                println!("Target: {}", intent.target);
                println!("Method: {}", intent.method);
                if let Some(destination) = intent.destination {
                    println!("Destination: {destination}");
                }
                println!(
                    "Execution ready: {}",
                    if intent.execution_ready { "yes" } else { "no" }
                );
                if let Some(host) = intent.host_resolution {
                    println!("Host: {} ({:?}) - {}", host.id, host.status, host.detail);
                }
                if let Some(identity) = intent.identity_resolution {
                    println!(
                        "Identity: {} ({:?}) - {}",
                        identity.id, identity.status, identity.detail
                    );
                }
                println!(
                    "Comparison: {}",
                    human_remote_comparison(&intent.comparison)
                );
                println!(
                    "Completed history directory: {}",
                    intent.completed_history_dir
                );
                println!("Upload:");
                for path in intent.upload {
                    println!("  {path}");
                }
                println!("Update:");
                for path in intent.update {
                    println!("  {path}");
                }
                println!("Delete:");
                for path in intent.delete {
                    println!("  {path}");
                }
                println!("Skip:");
                for path in intent.skip {
                    println!("  {path}");
                }
                if !intent.confirmations.is_empty() {
                    println!("Confirmations:");
                    for confirmation in intent.confirmations {
                        println!("  {confirmation}");
                    }
                }
                if !intent.blocked_reasons.is_empty() {
                    println!("Blocked reasons:");
                    for reason in intent.blocked_reasons {
                        println!("  {}: {}", reason.code, reason.message);
                    }
                }
            }
            0
        }
        Err(error) => print_command_error("publish", "transfer_intent", &error.to_string(), json),
    }
}

fn prepare_removable_execution(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    let context = publish_context(args);
    match prepare_removable_execution_with_context(Path::new(project), target, &context) {
        Ok(plan) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{}}}",
                    json_removable_execution_plan(&plan)
                );
            } else {
                println!("Removable execution plan");
                println!("Project: {}", plan.project_id);
                println!("Target: {}", plan.target);
                println!("Method: {}", plan.method);
                println!("Destination root: {}", plan.destination_root);
                println!(
                    "Execution ready: {}",
                    if plan.execution_ready { "yes" } else { "no" }
                );
                println!(
                    "Completed history directory: {}",
                    plan.completed_history_dir
                );
                print_removable_operations("Upload", &plan.upload);
                print_removable_operations("Update", &plan.update);
                print_removable_operations("Delete", &plan.delete);
                print_removable_operations("Skip", &plan.skip);
                if !plan.confirmations.is_empty() {
                    println!("Confirmations:");
                    for confirmation in plan.confirmations {
                        println!("  {confirmation}");
                    }
                }
                if !plan.blocked_reasons.is_empty() {
                    println!("Blocked reasons:");
                    for reason in plan.blocked_reasons {
                        println!("  {}: {}", reason.code, reason.message);
                    }
                }
            }
            0
        }
        Err(error) => print_command_error(
            "publish",
            "prepare_removable_execution",
            &error.to_string(),
            json,
        ),
    }
}

fn execute_removable(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };
    let Some(date) = option_value(args, "--date") else {
        return usage_error("missing --date");
    };

    let service = PublishService;
    match service.execute_removable(ExecuteRemovableRequest {
        project_path: PathBuf::from(project),
        target: target.to_string(),
        remote_state_path: option_value(args, "--remote-state").map(PathBuf::from),
        date: date.to_string(),
        confirm_transfer: args.contains(&"--confirm-transfer"),
    }) {
        Ok(executed) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{}}}",
                    json_execute_removable_response(&executed)
                );
            } else {
                println!("Removable execution completed");
                println!("Project: {}", executed.plan.project_id);
                println!("Target: {}", executed.plan.target);
                println!("Method: {}", executed.plan.method);
                println!("Destination root: {}", executed.plan.destination_root);
                println!("Transfer result: {}", executed.result.transfer_result);
                println!(
                    "Verification result: {}",
                    executed.result.verification_result
                );
                println!("Completed history: {}", executed.history_path.display());
                println!("Files:");
                for file in executed.result.files {
                    println!("  {} {}: {}", file.result, file.action, file.project_path);
                }
            }
            0
        }
        Err(error) => print_command_error("publish", "execute_removable", &error.to_string(), json),
    }
}

#[derive(Debug)]
struct CompletedHistoryInputs<'a> {
    project: &'a str,
    target: &'a str,
    date: &'a str,
    transfer_result: &'a str,
    verification_result: &'a str,
    rollback_available: bool,
    rollback_notes: &'a str,
}

fn publish_context(args: &[&str]) -> PublishContext {
    PublishContext {
        hosts_root: option_value(args, "--hosts").map(PathBuf::from),
        identities_root: option_value(args, "--identities").map(PathBuf::from),
        remote_state_path: option_value(args, "--remote-state").map(PathBuf::from),
    }
}

fn dry_run(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };

    let context = publish_context(args);

    match dry_run_publish_with_context(Path::new(project), target, &context) {
        Ok(plan) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"destination\":{},\"blocked\":{},\"host_resolution\":{},\"identity_resolution\":{},\"feed\":{},\"comparison\":{},\"changes\":{{\"upload\":[{}],\"update\":[{}],\"delete\":[{}],\"skip\":[{}]}},\"verification\":{{\"checks\":[{}]}},\"confirmations\":[{}]}}}}",
                    escape_json(&plan.project_id),
                    escape_json(&plan.target),
                    escape_json(&plan.method),
                    json_optional_string(plan.destination.as_deref()),
                    plan.blocked,
                    json_resolution(plan.host_resolution.as_ref()),
                    json_resolution(plan.identity_resolution.as_ref()),
                    json_feed_state(&plan.feed),
                    json_remote_comparison(&plan.comparison),
                    json_string_array(&plan.upload),
                    json_string_array(&plan.update),
                    json_string_array(&plan.delete),
                    json_string_array(&plan.skip),
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
                println!("Feed: {}", human_feed_state(&plan.feed));
                println!("Comparison: {}", human_remote_comparison(&plan.comparison));
                if !plan.feed.invalid_entry_diagnostics.is_empty() {
                    println!("Feed diagnostics:");
                    for diagnostic in &plan.feed.invalid_entry_diagnostics {
                        println!("  {}", diagnostic.path);
                        for issue in &diagnostic.issues {
                            println!("    {issue}");
                        }
                    }
                }
                println!("Upload:");
                for path in &plan.upload {
                    println!("  {path}");
                }
                println!("Update:");
                for path in &plan.update {
                    println!("  {path}");
                }
                println!("Delete:");
                for path in &plan.delete {
                    println!("  {path}");
                }
                println!("Skip:");
                for path in &plan.skip {
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

fn completed_history(args: &[&str], json: bool) -> i32 {
    let inputs = match completed_history_inputs(args) {
        Ok(inputs) => inputs,
        Err(message) => return usage_error(&message),
    };

    let context = publish_context(args);
    match dry_run_publish_with_context(Path::new(inputs.project), inputs.target, &context) {
        Ok(plan) => {
            let record = completed_history_record_from_plan(&plan, &inputs);
            let toml = record.to_toml();
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"transfer_result\":\"{}\",\"verification_result\":\"{}\",\"files\":[{}],\"record_toml\":\"{}\"}}}}",
                    escape_json(&record.project_id),
                    escape_json(&record.target),
                    escape_json(&record.transfer_result),
                    escape_json(&record.verification_result),
                    json_history_files(&record),
                    escape_json(&toml)
                );
            } else {
                print!("{toml}");
            }
            0
        }
        Err(error) => print_command_error("publish", "completed_history", &error.to_string(), json),
    }
}

fn planned_history(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };
    let Some(date) = option_value(args, "--date") else {
        return usage_error("missing --date");
    };

    let context = publish_context(args);
    match dry_run_publish_with_context(Path::new(project), target, &context) {
        Ok(plan) => {
            let record = PublicationHistoryRecord::planned_from_dry_run(&plan, date);
            let toml = record.to_toml();
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"transfer_result\":\"{}\",\"verification_result\":\"{}\",\"files\":[{}],\"record_toml\":\"{}\"}}}}",
                    escape_json(&record.project_id),
                    escape_json(&record.target),
                    escape_json(&record.transfer_result),
                    escape_json(&record.verification_result),
                    json_history_files(&record),
                    escape_json(&toml)
                );
            } else {
                print!("{toml}");
            }
            0
        }
        Err(error) => print_command_error("publish", "planned_history", &error.to_string(), json),
    }
}

fn save_completed_history(args: &[&str], json: bool) -> i32 {
    let inputs = match completed_history_inputs(args) {
        Ok(inputs) => inputs,
        Err(message) => return usage_error(&message),
    };

    let context = publish_context(args);
    match dry_run_publish_with_context(Path::new(inputs.project), inputs.target, &context) {
        Ok(plan) => {
            let record = completed_history_record_from_plan(&plan, &inputs);
            match write_completed_history_record(Path::new(inputs.project), &record) {
                Ok(output_path) => {
                    if json {
                        println!(
                            "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"output_path\":\"{}\",\"transfer_result\":\"{}\",\"verification_result\":\"{}\",\"files\":[{}]}}}}",
                            escape_json(&record.project_id),
                            escape_json(&record.target),
                            escape_json(&output_path.display().to_string()),
                            escape_json(&record.transfer_result),
                            escape_json(&record.verification_result),
                            json_history_files(&record)
                        );
                    } else {
                        println!("Saved completed history record: {}", output_path.display());
                    }
                    0
                }
                Err(error) => print_command_error(
                    "publish",
                    "save_completed_history",
                    &error.to_string(),
                    json,
                ),
            }
        }
        Err(error) => print_command_error(
            "publish",
            "save_completed_history",
            &error.to_string(),
            json,
        ),
    }
}

fn save_planned_history_preview(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(target) = option_value(args, "--target") else {
        return usage_error("missing --target");
    };
    let Some(date) = option_value(args, "--date") else {
        return usage_error("missing --date");
    };

    let context = publish_context(args);
    match dry_run_publish_with_context(Path::new(project), target, &context) {
        Ok(plan) => {
            let record = PublicationHistoryRecord::planned_from_dry_run(&plan, date);
            match write_planned_history_preview(Path::new(project), &record) {
                Ok(output_path) => {
                    if json {
                        println!(
                            "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project\":\"{}\",\"target\":\"{}\",\"output_path\":\"{}\",\"transfer_result\":\"{}\",\"verification_result\":\"{}\",\"files\":[{}]}}}}",
                            escape_json(&record.project_id),
                            escape_json(&record.target),
                            escape_json(&output_path.display().to_string()),
                            escape_json(&record.transfer_result),
                            escape_json(&record.verification_result),
                            json_history_files(&record)
                        );
                    } else {
                        println!("Saved planned history preview: {}", output_path.display());
                    }
                    0
                }
                Err(error) => print_command_error(
                    "publish",
                    "save_planned_history_preview",
                    &error.to_string(),
                    json,
                ),
            }
        }
        Err(error) => print_command_error(
            "publish",
            "save_planned_history_preview",
            &error.to_string(),
            json,
        ),
    }
}

fn list_completed_history_files(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };

    match list_completed_history_records(Path::new(project)) {
        Ok(records) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project_path\":\"{}\",\"records\":[{}]}}}}",
                    escape_json(project),
                    json_completed_history_records(&records)
                );
            } else if records.is_empty() {
                println!("No completed history records");
            } else {
                println!("Completed history records");
                for record in records {
                    println!(
                        "  {} ({} bytes, modified {})",
                        record.path.display(),
                        record.size_bytes,
                        record.modified_unix
                    );
                }
            }
            0
        }
        Err(error) => print_command_error(
            "publish",
            "list_completed_history",
            &error.to_string(),
            json,
        ),
    }
}

fn list_planned_history_preview_files(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };

    match list_planned_history_previews(Path::new(project)) {
        Ok(previews) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project_path\":\"{}\",\"previews\":[{}]}}}}",
                    escape_json(project),
                    json_planned_history_previews(&previews)
                );
            } else if previews.is_empty() {
                println!("No planned history previews");
            } else {
                println!("Planned history previews");
                for preview in previews {
                    println!(
                        "  {} ({} bytes, modified {})",
                        preview.path.display(),
                        preview.size_bytes,
                        preview.modified_unix
                    );
                }
            }
            0
        }
        Err(error) => print_command_error(
            "publish",
            "list_planned_history_previews",
            &error.to_string(),
            json,
        ),
    }
}

fn read_completed_history_file(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(record) = option_value(args, "--record") else {
        return usage_error("missing --record");
    };

    match read_completed_history_record(Path::new(project), Path::new(record)) {
        Ok(detail) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project_path\":\"{}\",{}}}}}",
                    escape_json(project),
                    json_completed_history_detail(&detail)
                );
            } else {
                print!("{}", detail.record_toml);
            }
            0
        }
        Err(error) => print_command_error(
            "publish",
            "read_completed_history",
            &error.to_string(),
            json,
        ),
    }
}

fn read_planned_history_preview_file(args: &[&str], json: bool) -> i32 {
    let Some(project) = option_value(args, "--project") else {
        return usage_error("missing --project");
    };
    let Some(preview) = option_value(args, "--preview") else {
        return usage_error("missing --preview");
    };

    match read_planned_history_preview(Path::new(project), Path::new(preview)) {
        Ok(detail) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project_path\":\"{}\",{}}}}}",
                    escape_json(project),
                    json_planned_history_preview_detail(&detail)
                );
            } else {
                print!("{}", detail.record_toml);
            }
            0
        }
        Err(error) => print_command_error(
            "publish",
            "read_planned_history_preview",
            &error.to_string(),
            json,
        ),
    }
}

fn json_planned_history_preview_detail(detail: &PlannedHistoryPreviewDetail) -> String {
    format!(
        "\"path\":\"{}\",\"filename\":\"{}\",\"modified_unix\":{},\"size_bytes\":{},\"record_toml\":\"{}\"",
        escape_json(&detail.entry.path.display().to_string()),
        escape_json(&detail.entry.filename),
        detail.entry.modified_unix,
        detail.entry.size_bytes,
        escape_json(&detail.record_toml)
    )
}

fn json_completed_history_detail(detail: &CompletedHistoryDetail) -> String {
    format!(
        "\"path\":\"{}\",\"filename\":\"{}\",\"modified_unix\":{},\"size_bytes\":{},\"record_toml\":\"{}\"",
        escape_json(&detail.entry.path.display().to_string()),
        escape_json(&detail.entry.filename),
        detail.entry.modified_unix,
        detail.entry.size_bytes,
        escape_json(&detail.record_toml)
    )
}

fn json_planned_history_previews(previews: &[PlannedHistoryPreviewEntry]) -> String {
    previews
        .iter()
        .map(|preview| {
            format!(
                "{{\"path\":\"{}\",\"filename\":\"{}\",\"modified_unix\":{},\"size_bytes\":{}}}",
                escape_json(&preview.path.display().to_string()),
                escape_json(&preview.filename),
                preview.modified_unix,
                preview.size_bytes
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_completed_history_records(records: &[CompletedHistoryEntry]) -> String {
    records
        .iter()
        .map(|record| {
            format!(
                "{{\"path\":\"{}\",\"filename\":\"{}\",\"modified_unix\":{},\"size_bytes\":{}}}",
                escape_json(&record.path.display().to_string()),
                escape_json(&record.filename),
                record.modified_unix,
                record.size_bytes
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_history_files(record: &PublicationHistoryRecord) -> String {
    record
        .files
        .iter()
        .map(|file| {
            format!(
                "{{\"path\":\"{}\",\"action\":\"{}\"}}",
                escape_json(&file.path),
                escape_json(&file.action)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_publish_validation_issues(issues: &[PublishValidationIssue]) -> String {
    issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"code\":\"{}\",\"message\":\"{}\",\"path\":{}}}",
                escape_json(issue.code),
                escape_json(&issue.message),
                json_optional_string(issue.path.as_deref())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_transfer_intent(intent: &TransferIntent) -> String {
    format!(
        "{{\"project\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"destination\":{},\"execution_ready\":{},\"blocked_reasons\":[{}],\"confirmations\":[{}],\"host_resolution\":{},\"identity_resolution\":{},\"comparison\":{},\"changes\":{{\"upload\":[{}],\"update\":[{}],\"delete\":[{}],\"skip\":[{}]}},\"history\":{{\"completed_directory\":\"{}\"}}}}",
        escape_json(&intent.project_id),
        escape_json(&intent.target),
        escape_json(&intent.method),
        json_optional_string(intent.destination.as_deref()),
        intent.execution_ready,
        json_publish_validation_issues(&intent.blocked_reasons),
        json_string_array(&intent.confirmations),
        json_resolution(intent.host_resolution.as_ref()),
        json_resolution(intent.identity_resolution.as_ref()),
        json_remote_comparison(&intent.comparison),
        json_string_array(&intent.upload),
        json_string_array(&intent.update),
        json_string_array(&intent.delete),
        json_string_array(&intent.skip),
        escape_json(&intent.completed_history_dir)
    )
}

fn json_removable_execution_plan(plan: &RemovableExecutionPlan) -> String {
    format!(
        "{{\"project\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"destination_root\":\"{}\",\"execution_ready\":{},\"blocked_reasons\":[{}],\"confirmations\":[{}],\"operations\":{{\"upload\":[{}],\"update\":[{}],\"delete\":[{}],\"skip\":[{}]}},\"history\":{{\"completed_directory\":\"{}\"}}}}",
        escape_json(&plan.project_id),
        escape_json(&plan.target),
        escape_json(&plan.method),
        escape_json(&plan.destination_root),
        plan.execution_ready,
        json_publish_validation_issues(&plan.blocked_reasons),
        json_string_array(&plan.confirmations),
        json_removable_operations(&plan.upload),
        json_removable_operations(&plan.update),
        json_removable_operations(&plan.delete),
        json_removable_operations(&plan.skip),
        escape_json(&plan.completed_history_dir)
    )
}

fn json_removable_operations(operations: &[RemovableExecutionOperation]) -> String {
    operations
        .iter()
        .map(|operation| {
            format!(
                "{{\"project_path\":\"{}\",\"source_path\":{},\"destination_path\":\"{}\"}}",
                escape_json(&operation.project_path),
                json_optional_string(operation.source_path.as_deref()),
                escape_json(&operation.destination_path)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_execute_removable_response(executed: &ExecuteRemovableResponse) -> String {
    let record_toml = executed.record.to_toml();
    format!(
        "{{\"project\":\"{}\",\"target\":\"{}\",\"method\":\"{}\",\"destination_root\":\"{}\",\"transfer_result\":\"{}\",\"verification_result\":\"{}\",\"files\":[{}],\"history\":{{\"completed_path\":\"{}\",\"record_toml\":\"{}\"}}}}",
        escape_json(&executed.plan.project_id),
        escape_json(&executed.plan.target),
        escape_json(&executed.plan.method),
        escape_json(&executed.plan.destination_root),
        escape_json(&executed.result.transfer_result),
        escape_json(&executed.result.verification_result),
        json_removable_file_results(&executed.result.files),
        escape_json(&executed.history_path.display().to_string()),
        escape_json(&record_toml)
    )
}

fn json_removable_file_results(files: &[RemovableExecutionFileResult]) -> String {
    files
        .iter()
        .map(|file| {
            format!(
                "{{\"project_path\":\"{}\",\"source_path\":{},\"destination_path\":\"{}\",\"action\":\"{}\",\"result\":\"{}\",\"bytes\":{}}}",
                escape_json(&file.project_path),
                json_optional_string(file.source_path.as_deref()),
                escape_json(&file.destination_path),
                escape_json(&file.action),
                escape_json(&file.result),
                file.bytes
                    .map(|bytes| bytes.to_string())
                    .unwrap_or_else(|| "null".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn print_removable_operations(label: &str, operations: &[RemovableExecutionOperation]) {
    println!("{label}:");
    for operation in operations {
        match &operation.source_path {
            Some(source) => println!(
                "  {}: {} -> {}",
                operation.project_path, source, operation.destination_path
            ),
            None => println!(
                "  {}: {}",
                operation.project_path, operation.destination_path
            ),
        }
    }
}

fn completed_history_record_from_plan(
    plan: &waystone_publish_plan::PublishDryRun,
    inputs: &CompletedHistoryInputs,
) -> PublicationHistoryRecord {
    PublicationHistoryRecord::completed_from_dry_run(
        plan,
        inputs.date,
        CompletedHistoryOptions {
            transfer_result: inputs.transfer_result.to_string(),
            verification_result: inputs.verification_result.to_string(),
            rollback_available: inputs.rollback_available,
            rollback_notes: inputs.rollback_notes.to_string(),
        },
    )
}

fn completed_history_inputs<'a>(args: &'a [&str]) -> Result<CompletedHistoryInputs<'a>, String> {
    let project = required_option(args, "--project")?;
    let target = required_option(args, "--target")?;
    let date = required_option(args, "--date")?;
    let transfer_result = required_option(args, "--transfer-result")?;
    let verification_result = required_option(args, "--verification-result")?;
    let rollback_available = parse_bool_option(required_option(args, "--rollback-available")?)?;
    let rollback_notes = required_option(args, "--rollback-notes")?;

    validate_option_value(
        "--transfer-result",
        transfer_result,
        &["completed", "failed", "skipped"],
    )?;
    validate_option_value(
        "--verification-result",
        verification_result,
        &["not-run", "passed", "failed"],
    )?;

    Ok(CompletedHistoryInputs {
        project,
        target,
        date,
        transfer_result,
        verification_result,
        rollback_available,
        rollback_notes,
    })
}

fn write_new_file(path: &Path, content: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {}", path.display(), error))?;
    file.write_all(content)
        .map_err(|error| format!("{}: {}", path.display(), error))
}

fn print_help() {
    println!("Usage:");
    println!(
        "  publish --export-remote-state --project PATH --target NAME [--output PATH] [--json]"
    );
    println!("  publish --inspect-remote-state --remote-state PATH [--json]");
    println!("  publish --execute-removable --project PATH --target NAME --date DATE --confirm-transfer [--remote-state PATH] [--json]");
    println!("  publish --prepare-removable-execution --project PATH --target NAME [--remote-state PATH] [--json]");
    println!("  publish --transfer-intent --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --validate --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --dry-run --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --planned-history --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --save-planned-history-preview --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --list-planned-history-previews --project PATH [--json]");
    println!("  publish --read-planned-history-preview --project PATH --preview PATH [--json]");
    println!("  publish --completed-history --project PATH --target NAME --date DATE --transfer-result completed|failed|skipped --verification-result not-run|passed|failed --rollback-available true|false --rollback-notes TEXT [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --save-completed-history --project PATH --target NAME --date DATE --transfer-result completed|failed|skipped --verification-result not-run|passed|failed --rollback-available true|false --rollback-notes TEXT [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]");
    println!("  publish --list-completed-history --project PATH [--json]");
    println!("  publish --read-completed-history --project PATH --record PATH [--json]");
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

fn required_option<'a>(args: &'a [&str], option: &str) -> Result<&'a str, String> {
    option_value(args, option).ok_or_else(|| format!("missing {option}"))
}

fn parse_bool_option(value: &str) -> Result<bool, String> {
    match value {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err("invalid --rollback-available (use true or false)".to_string()),
    }
}

fn validate_option_value(option: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "invalid {option} value '{value}' (allowed: {})",
            allowed.join(", ")
        ))
    }
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

fn json_feed_state(feed: &FeedPublicationState) -> String {
    format!(
        "{{\"configured\":{},\"enabled\":{},\"type\":{},\"path\":{},\"exists\":{},\"prepared_entries\":{},\"invalid_entries\":{},\"invalid_entry_diagnostics\":[{}]}}",
        feed.configured,
        feed.enabled,
        json_optional_string(feed.feed_type.as_deref()),
        json_optional_string(feed.path.as_deref()),
        feed.exists,
        feed.prepared_entries,
        feed.invalid_entries,
        json_feed_diagnostics(&feed.invalid_entry_diagnostics)
    )
}

fn json_remote_comparison(comparison: &RemoteComparisonState) -> String {
    format!(
        "{{\"configured\":{},\"source\":{},\"remote_paths\":{}}}",
        comparison.configured,
        json_optional_string(comparison.source.as_deref()),
        comparison.remote_paths
    )
}

fn json_remote_state_manifest_identity(manifest: &RemoteStateManifest) -> String {
    format!(
        "\"project\":{},\"target\":{},\"source\":{}",
        json_optional_string(manifest.project_id.as_deref()),
        json_optional_string(manifest.target.as_deref()),
        json_optional_string(manifest.source.as_deref())
    )
}

fn json_feed_diagnostics(diagnostics: &[FeedEntryDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{{\"path\":\"{}\",\"issues\":[{}]}}",
                escape_json(&diagnostic.path),
                json_string_array(&diagnostic.issues)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn human_feed_state(feed: &FeedPublicationState) -> String {
    if !feed.configured {
        return "not configured".to_string();
    }

    format!(
        "{} ({}, {} prepared entries, {} invalid entries)",
        feed.path.as_deref().unwrap_or("no path"),
        if feed.exists { "exists" } else { "missing" },
        feed.prepared_entries,
        feed.invalid_entries
    )
}

fn human_remote_comparison(comparison: &RemoteComparisonState) -> String {
    if !comparison.configured {
        return "not configured".to_string();
    }

    format!(
        "{} ({} remote paths)",
        comparison.source.as_deref().unwrap_or("unknown source"),
        comparison.remote_paths
    )
}
