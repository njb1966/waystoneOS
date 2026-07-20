use std::env;
use std::path::Path;
use std::process;
use waystone_audio_metadata::{list_recordings, load_audio_metadata, validate_audio_metadata};
use waystone_audio_service::{
    AttachRecordingRequest, AudioService, CaptureRecordingRequest, ExportOpusRequest,
    GenerateFeedRequest, PrepareFeedEntryRequest, UpdateFeedEntryRequest, UpdateRecordingRequest,
    ValidateFeedEntryRequest, ValidatePublicationRequest,
};
use waystone_cli_output::{escape_json, json_optional_string, print_command_error};
use waystone_project_format::load_manifest;

struct AttachArgs<'a> {
    project: &'a str,
    id: &'a str,
    title: &'a str,
    master: &'a str,
    published: &'a str,
    feed: &'a str,
    entry_id: &'a str,
    mime_type: &'a str,
}

struct PrepareFeedEntryArgs<'a> {
    project: &'a str,
    recording_id: &'a str,
    updated: &'a str,
    summary: &'a str,
}

struct UpdateArgs<'a> {
    project: &'a str,
    recording_id: &'a str,
    title: &'a str,
    master: &'a str,
    published: &'a str,
    feed: &'a str,
    entry_id: &'a str,
    mime_type: &'a str,
}

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
        ["attach", project, id, title, master, published, feed, entry_id, mime_type] => attach(
            AttachArgs {
                project,
                id,
                title,
                master,
                published,
                feed,
                entry_id,
                mime_type,
            },
            json,
        ),
        ["update", project, recording_id, title, master, published, feed, entry_id, mime_type] => {
            update(
                UpdateArgs {
                    project,
                    recording_id,
                    title,
                    master,
                    published,
                    feed,
                    entry_id,
                    mime_type,
                },
                json,
            )
        }
        ["export-opus", project, master, published, preset] => {
            export_opus(project, master, published, preset, json)
        }
        ["capture", project, master, duration_seconds, input_format, input] => {
            capture(project, master, duration_seconds, input_format, input, json)
        }
        ["prepare-feed-entry", project, recording_id, updated, summary] => prepare_feed_entry(
            PrepareFeedEntryArgs {
                project,
                recording_id,
                updated,
                summary,
            },
            json,
        ),
        ["update-feed-entry", project, recording_id, updated, summary] => update_feed_entry(
            PrepareFeedEntryArgs {
                project,
                recording_id,
                updated,
                summary,
            },
            json,
        ),
        ["validate-publication", project, recording_id] => {
            validate_publication(project, recording_id, json)
        }
        ["validate-feed-entry", project, recording_id] => {
            validate_feed_entry(project, recording_id, json)
        }
        ["generate-feed", project] => generate_feed(project, json),
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

fn attach(args: AttachArgs<'_>, json: bool) -> i32 {
    let metadata_root = match project_audio_metadata_root(args.project, "attach", json) {
        Ok(metadata_root) => metadata_root,
        Err(exit_code) => return exit_code,
    };

    let service = AudioService;
    match service.attach_recording(AttachRecordingRequest {
        project_root: Path::new(args.project).to_path_buf(),
        metadata_root,
        id: args.id.to_string(),
        title: args.title.to_string(),
        master: args.master.to_string(),
        published: args.published.to_string(),
        feed: args.feed.to_string(),
        entry_id: args.entry_id.to_string(),
        mime_type: args.mime_type.to_string(),
    }) {
        Ok(attached) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"title\":\"{}\",\"metadata_path\":\"{}\",\"metadata_relative_path\":\"{}\",\"master\":\"{}\",\"published\":\"{}\",\"feed\":\"{}\",\"entry_id\":\"{}\",\"mime_type\":\"{}\"}}}}",
                    escape_json(&attached.id),
                    escape_json(&attached.title),
                    escape_json(&attached.metadata_path.display().to_string()),
                    escape_json(&attached.metadata_relative_path),
                    escape_json(&attached.master),
                    escape_json(&attached.published),
                    escape_json(&attached.feed),
                    escape_json(&attached.entry_id),
                    escape_json(&attached.mime_type)
                );
            } else {
                println!("Attached recording: {}", attached.id);
                println!("Metadata: {}", attached.metadata_path.display());
                println!("Master: {}", attached.master);
                println!("Published: {}", attached.published);
                println!("Feed: {}", attached.feed);
            }
            0
        }
        Err(error) => print_command_error("record", "attach", &error.to_string(), json),
    }
}

fn update(args: UpdateArgs<'_>, json: bool) -> i32 {
    let metadata_root = match project_audio_metadata_root(args.project, "update", json) {
        Ok(metadata_root) => metadata_root,
        Err(exit_code) => return exit_code,
    };
    let recording_metadata_path = Path::new(args.project)
        .join(metadata_root)
        .join(format!("{}.toml", args.recording_id));

    let service = AudioService;
    match service.update_recording(UpdateRecordingRequest {
        project_root: Path::new(args.project).to_path_buf(),
        recording_metadata_path,
        title: args.title.to_string(),
        master: args.master.to_string(),
        published: args.published.to_string(),
        feed: args.feed.to_string(),
        entry_id: args.entry_id.to_string(),
        mime_type: args.mime_type.to_string(),
    }) {
        Ok(updated) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"title\":\"{}\",\"metadata_path\":\"{}\",\"metadata_relative_path\":\"{}\",\"master\":\"{}\",\"published\":\"{}\",\"feed\":\"{}\",\"entry_id\":\"{}\",\"mime_type\":\"{}\"}}}}",
                    escape_json(&updated.id),
                    escape_json(&updated.title),
                    escape_json(&updated.metadata_path.display().to_string()),
                    escape_json(&updated.metadata_relative_path),
                    escape_json(&updated.master),
                    escape_json(&updated.published),
                    escape_json(&updated.feed),
                    escape_json(&updated.entry_id),
                    escape_json(&updated.mime_type)
                );
            } else {
                println!("Updated recording: {}", updated.id);
                println!("Metadata: {}", updated.metadata_path.display());
                println!("Master: {}", updated.master);
                println!("Published: {}", updated.published);
                println!("Feed: {}", updated.feed);
            }
            0
        }
        Err(error) => print_command_error("record", "update", &error.to_string(), json),
    }
}

fn export_opus(project: &str, master: &str, published: &str, preset: &str, json: bool) -> i32 {
    if let Err(error) = load_manifest(Path::new(project)) {
        return print_command_error("record", "export-opus", &error.to_string(), json);
    }

    let service = AudioService;
    match service.export_opus(ExportOpusRequest {
        project_root: Path::new(project).to_path_buf(),
        master: master.to_string(),
        published: published.to_string(),
        preset: preset.to_string(),
    }) {
        Ok(exported) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"master\":\"{}\",\"published\":\"{}\",\"output_path\":\"{}\",\"output_relative_path\":\"{}\",\"preset\":\"{}\",\"mime_type\":\"{}\",\"engine\":\"{}\"}}}}",
                    escape_json(&exported.master),
                    escape_json(&exported.published),
                    escape_json(&exported.output_path.display().to_string()),
                    escape_json(&exported.output_relative_path),
                    escape_json(&exported.preset),
                    escape_json(&exported.mime_type),
                    escape_json(&exported.engine)
                );
            } else {
                println!("Exported Opus publication copy: {}", exported.published);
                println!("Engine: {}", exported.engine);
                println!("Preset: {}", exported.preset);
                println!("Output: {}", exported.output_path.display());
            }
            0
        }
        Err(error) => print_command_error("record", "export-opus", &error.to_string(), json),
    }
}

fn capture(
    project: &str,
    master: &str,
    duration_seconds: &str,
    input_format: &str,
    input: &str,
    json: bool,
) -> i32 {
    let masters_root = match project_audio_masters_root(project, "capture", json) {
        Ok(masters_root) => masters_root,
        Err(exit_code) => return exit_code,
    };
    let duration_seconds = match duration_seconds.parse::<u32>() {
        Ok(duration_seconds) => duration_seconds,
        Err(error) => {
            return print_command_error(
                "record",
                "capture",
                &format!("duration seconds must be an unsigned integer: {error}"),
                json,
            );
        }
    };

    let service = AudioService;
    match service.capture_recording(CaptureRecordingRequest {
        project_root: Path::new(project).to_path_buf(),
        masters_root,
        master: master.to_string(),
        duration_seconds,
        input_format: input_format.to_string(),
        input: input.to_string(),
    }) {
        Ok(captured) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"master\":\"{}\",\"output_path\":\"{}\",\"output_relative_path\":\"{}\",\"duration_seconds\":{},\"channels\":{},\"sample_rate\":{},\"format\":\"{}\",\"engine\":\"{}\"}}}}",
                    escape_json(&captured.master),
                    escape_json(&captured.output_path.display().to_string()),
                    escape_json(&captured.output_relative_path),
                    captured.duration_seconds,
                    captured.channels,
                    captured.sample_rate,
                    escape_json(&captured.format),
                    escape_json(&captured.engine)
                );
            } else {
                println!("Captured recording master: {}", captured.master);
                println!("Engine: {}", captured.engine);
                println!("Duration: {} seconds", captured.duration_seconds);
                println!("Output: {}", captured.output_path.display());
            }
            0
        }
        Err(error) => print_command_error("record", "capture", &error.to_string(), json),
    }
}

fn prepare_feed_entry(args: PrepareFeedEntryArgs<'_>, json: bool) -> i32 {
    let metadata_root = match project_audio_metadata_root(args.project, "prepare-feed-entry", json)
    {
        Ok(metadata_root) => metadata_root,
        Err(exit_code) => return exit_code,
    };
    let recording_metadata_path = Path::new(args.project)
        .join(metadata_root)
        .join(format!("{}.toml", args.recording_id));

    let service = AudioService;
    match service.prepare_feed_entry(PrepareFeedEntryRequest {
        project_root: Path::new(args.project).to_path_buf(),
        recording_metadata_path,
        updated: args.updated.to_string(),
        summary: args.summary.to_string(),
    }) {
        Ok(prepared) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"recording_id\":\"{}\",\"title\":\"{}\",\"entry_id\":\"{}\",\"feed\":\"{}\",\"output_path\":\"{}\",\"output_relative_path\":\"{}\",\"published\":\"{}\",\"mime_type\":\"{}\",\"updated\":\"{}\"}}}}",
                    escape_json(&prepared.recording_id),
                    escape_json(&prepared.title),
                    escape_json(&prepared.entry_id),
                    escape_json(&prepared.feed),
                    escape_json(&prepared.output_path.display().to_string()),
                    escape_json(&prepared.output_relative_path),
                    escape_json(&prepared.published),
                    escape_json(&prepared.mime_type),
                    escape_json(&prepared.updated)
                );
            } else {
                println!("Prepared feed entry: {}", prepared.recording_id);
                println!("Metadata: {}", prepared.output_path.display());
                println!("Published: {}", prepared.published);
                println!("Feed: {}", prepared.feed);
            }
            0
        }
        Err(error) => print_command_error("record", "prepare-feed-entry", &error.to_string(), json),
    }
}

fn update_feed_entry(args: PrepareFeedEntryArgs<'_>, json: bool) -> i32 {
    let metadata_root = match project_audio_metadata_root(args.project, "update-feed-entry", json) {
        Ok(metadata_root) => metadata_root,
        Err(exit_code) => return exit_code,
    };
    let recording_metadata_path = Path::new(args.project)
        .join(metadata_root)
        .join(format!("{}.toml", args.recording_id));

    let service = AudioService;
    match service.update_feed_entry(UpdateFeedEntryRequest {
        project_root: Path::new(args.project).to_path_buf(),
        recording_metadata_path,
        updated: args.updated.to_string(),
        summary: args.summary.to_string(),
    }) {
        Ok(updated) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"recording_id\":\"{}\",\"title\":\"{}\",\"entry_id\":\"{}\",\"feed\":\"{}\",\"output_path\":\"{}\",\"output_relative_path\":\"{}\",\"published\":\"{}\",\"mime_type\":\"{}\",\"updated\":\"{}\"}}}}",
                    escape_json(&updated.recording_id),
                    escape_json(&updated.title),
                    escape_json(&updated.entry_id),
                    escape_json(&updated.feed),
                    escape_json(&updated.output_path.display().to_string()),
                    escape_json(&updated.output_relative_path),
                    escape_json(&updated.published),
                    escape_json(&updated.mime_type),
                    escape_json(&updated.updated)
                );
            } else {
                println!("Updated feed entry: {}", updated.recording_id);
                println!("Metadata: {}", updated.output_path.display());
                println!("Published: {}", updated.published);
                println!("Feed: {}", updated.feed);
            }
            0
        }
        Err(error) => print_command_error("record", "update-feed-entry", &error.to_string(), json),
    }
}

fn validate_publication(project: &str, recording_id: &str, json: bool) -> i32 {
    let metadata_root = match project_audio_metadata_root(project, "validate-publication", json) {
        Ok(metadata_root) => metadata_root,
        Err(exit_code) => return exit_code,
    };
    let recording_metadata_path = Path::new(project)
        .join(metadata_root)
        .join(format!("{recording_id}.toml"));

    let service = AudioService;
    match service.validate_publication(ValidatePublicationRequest {
        project_root: Path::new(project).to_path_buf(),
        recording_metadata_path,
    }) {
        Ok(report) => print_validation_result("publication", report, json),
        Err(error) => {
            print_command_error("record", "validate-publication", &error.to_string(), json)
        }
    }
}

fn validate_feed_entry(project: &str, recording_id: &str, json: bool) -> i32 {
    let feed_entry_path = Path::new(project)
        .join("feeds/entries")
        .join(format!("{recording_id}.toml"));

    let service = AudioService;
    match service.validate_feed_entry(ValidateFeedEntryRequest {
        project_root: Path::new(project).to_path_buf(),
        feed_entry_path,
    }) {
        Ok(report) => print_validation_result("feed entry", report, json),
        Err(error) => {
            print_command_error("record", "validate-feed-entry", &error.to_string(), json)
        }
    }
}

fn generate_feed(project: &str, json: bool) -> i32 {
    let manifest = match load_manifest(Path::new(project)) {
        Ok(manifest) => manifest,
        Err(error) => {
            return print_command_error("record", "generate-feed", &error.to_string(), json);
        }
    };

    let Some(feed) = manifest.feed else {
        return print_command_error(
            "record",
            "generate-feed",
            "project feed is not configured",
            json,
        );
    };
    if !feed.enabled.unwrap_or(false) {
        return print_command_error(
            "record",
            "generate-feed",
            "project feed is not enabled",
            json,
        );
    }
    if feed.feed_type.as_deref().unwrap_or("atom").trim() != "atom" {
        return print_command_error(
            "record",
            "generate-feed",
            "only Atom feeds are supported",
            json,
        );
    }

    let Some(feed_path) = feed.path.filter(|path| !path.trim().is_empty()) else {
        return print_command_error(
            "record",
            "generate-feed",
            "project feed path is not configured",
            json,
        );
    };
    let title = feed
        .title
        .filter(|title| !title.trim().is_empty())
        .unwrap_or(manifest.project.name);

    let service = AudioService;
    match service.generate_feed(GenerateFeedRequest {
        project_root: Path::new(project).to_path_buf(),
        feed_path,
        title,
    }) {
        Ok(generated) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"feed_path\":\"{}\",\"feed_relative_path\":\"{}\",\"entries\":{},\"updated\":\"{}\"}}}}",
                    escape_json(&generated.feed_path.display().to_string()),
                    escape_json(&generated.feed_relative_path),
                    generated.entries,
                    escape_json(&generated.updated)
                );
            } else {
                println!("Generated feed: {}", generated.feed_path.display());
                println!("Entries: {}", generated.entries);
            }
            0
        }
        Err(error) => print_command_error("record", "generate-feed", &error.to_string(), json),
    }
}

fn project_audio_metadata_root(project: &str, command: &str, json: bool) -> Result<String, i32> {
    let manifest = load_manifest(Path::new(project))
        .map_err(|error| print_command_error("record", command, &error.to_string(), json))?;

    manifest
        .audio
        .and_then(|audio| audio.metadata)
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| {
            print_command_error(
                "record",
                command,
                "project audio metadata root is not configured",
                json,
            )
        })
}

fn project_audio_masters_root(project: &str, command: &str, json: bool) -> Result<String, i32> {
    let manifest = load_manifest(Path::new(project))
        .map_err(|error| print_command_error("record", command, &error.to_string(), json))?;

    manifest
        .audio
        .and_then(|audio| audio.masters)
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| {
            print_command_error(
                "record",
                command,
                "project audio masters root is not configured",
                json,
            )
        })
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
        Ok(report) => print_validation_result("recording", report, json),
        Err(error) => print_command_error("record", "validate", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  record capture [--json] PROJECT MASTER DURATION_SECONDS INPUT_FORMAT INPUT");
    println!("  record export-opus [--json] PROJECT MASTER PUBLISHED PRESET");
    println!("  record attach [--json] PROJECT ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE");
    println!(
        "  record update [--json] PROJECT RECORDING_ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE"
    );
    println!("  record prepare-feed-entry [--json] PROJECT RECORDING_ID UPDATED SUMMARY");
    println!("  record update-feed-entry [--json] PROJECT RECORDING_ID UPDATED SUMMARY");
    println!("  record validate-publication [--json] PROJECT RECORDING_ID");
    println!("  record validate-feed-entry [--json] PROJECT RECORDING_ID");
    println!("  record generate-feed [--json] PROJECT");
    println!("  record list [--json] ROOT");
    println!("  record inspect [--json] PATH");
    println!("  record validate [--json] PATH");
}

fn print_validation_result(
    label: &str,
    report: waystone_audio_metadata::ValidationReport,
    json: bool,
) -> i32 {
    print_validation(label, report.valid, &report.errors, &report.warnings, json);
    if report.valid {
        0
    } else {
        3
    }
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
