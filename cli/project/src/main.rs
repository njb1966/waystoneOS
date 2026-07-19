use std::env;
use std::path::{Path, PathBuf};
use std::process;
use waystone_cli_output::{escape_json, print_command_error};
use waystone_project_format::{
    add_removable_publish_target, create_project, inspect_project, list_projects, validate_project,
    AddRemovablePublishTargetOptions, ProjectCreateOptions,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = run(&args);
    process::exit(code);
}

fn run(args: &[String]) -> i32 {
    let json = args.iter().any(|arg| arg == "--json");
    let positional: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| *arg != "--json")
        .collect();

    match positional.as_slice() {
        ["create", parent, id, name, project_type] => create(parent, id, name, project_type, json),
        ["target", "add-removable", path, name, export_path] => {
            add_removable_target(path, name, export_path, json)
        }
        ["list", root] => list(root, json),
        ["inspect", path] => inspect(path, json),
        ["validate", path] => validate(path, json),
        ["help"] | ["--help"] | [] => {
            print_help();
            0
        }
        _ => {
            eprintln!("project: usage error");
            print_help();
            2
        }
    }
}

fn inspect(path: &str, json: bool) -> i32 {
    match inspect_project(Path::new(path)) {
        Ok(inspection) => {
            if json {
                let publish_targets = inspection
                    .publish_targets
                    .iter()
                    .map(|target| format!("\"{}\"", escape_json(target)))
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"id\":\"{}\",\"name\":\"{}\",\"type\":\"{}\",\"project_schema\":{},\"content_root\":\"{}\",\"content_index\":\"{}\",\"publish_targets\":[{}]}}}}",
                    escape_json(&inspection.id),
                    escape_json(&inspection.name),
                    escape_json(&inspection.project_type),
                    inspection.schema,
                    escape_json(&inspection.content_root),
                    escape_json(&inspection.content_index),
                    publish_targets
                );
            } else {
                println!("Project: {}", inspection.name);
                println!("ID: {}", inspection.id);
                println!("Type: {}", inspection.project_type);
                println!("Schema: {}", inspection.schema);
                println!(
                    "Content: {}/{}",
                    inspection.content_root, inspection.content_index
                );
                if inspection.publish_targets.is_empty() {
                    println!("Publish targets: none");
                } else {
                    println!("Publish targets: {}", inspection.publish_targets.join(", "));
                }
            }
            0
        }
        Err(error) => print_command_error("project", "inspect", &error.to_string(), json),
    }
}

fn create(parent: &str, id: &str, name: &str, project_type: &str, json: bool) -> i32 {
    let options = ProjectCreateOptions {
        parent: PathBuf::from(parent),
        id: id.to_string(),
        name: name.to_string(),
        project_type: project_type.to_string(),
        content_index: "index.gmi".to_string(),
        language: Some("en".to_string()),
        author: None,
    };

    match create_project(&options) {
        Ok(created) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"project_path\":\"{}\",\"project_schema\":{}}}}}",
                    escape_json(&created.project_path.display().to_string()),
                    created.schema
                );
            } else {
                println!("Created project: {}", created.project_path.display());
            }
            0
        }
        Err(error) => print_command_error("project", "create", &error.to_string(), json),
    }
}

fn add_removable_target(path: &str, name: &str, export_path: &str, json: bool) -> i32 {
    let options = AddRemovablePublishTargetOptions {
        project_root: PathBuf::from(path),
        name: name.to_string(),
        path: export_path.to_string(),
    };

    match add_removable_publish_target(&options) {
        Ok(()) => {
            if json {
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"name\":\"{}\",\"method\":\"removable\",\"path\":\"{}\"}}}}",
                    escape_json(name),
                    escape_json(export_path)
                );
            } else {
                println!("Added removable publish target: {name}");
            }
            0
        }
        Err(error) => {
            print_command_error("project", "target add-removable", &error.to_string(), json)
        }
    }
}

fn list(root: &str, json: bool) -> i32 {
    match list_projects(Path::new(root)) {
        Ok(projects) => {
            if json {
                let projects_json = projects
                    .iter()
                    .map(|project| {
                        format!(
                            "{{\"id\":\"{}\",\"name\":\"{}\",\"type\":\"{}\",\"schema\":{},\"path\":\"{}\"}}",
                            escape_json(&project.id),
                            escape_json(&project.name),
                            escape_json(&project.project_type),
                            project.schema,
                            escape_json(&project.path.display().to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"projects\":[{}]}}}}",
                    projects_json
                );
            } else if projects.is_empty() {
                println!("No projects found");
            } else {
                for project in projects {
                    println!(
                        "{}\t{}\t{}\t{}",
                        project.id,
                        project.project_type,
                        project.schema,
                        project.path.display()
                    );
                }
            }
            0
        }
        Err(error) => print_command_error("project", "list", &error.to_string(), json),
    }
}

fn validate(path: &str, json: bool) -> i32 {
    match validate_project(Path::new(path)) {
        Ok(report) => {
            if json {
                let errors = report
                    .errors
                    .iter()
                    .map(|issue| {
                        format!(
                            "{{\"code\":\"{}\",\"message\":\"{}\"}}",
                            issue.code,
                            escape_json(&issue.message)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let warnings = report
                    .warnings
                    .iter()
                    .map(|issue| {
                        format!(
                            "{{\"code\":\"{}\",\"message\":\"{}\"}}",
                            issue.code,
                            escape_json(&issue.message)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                println!(
                    "{{\"status\":\"ok\",\"schema\":1,\"data\":{{\"valid\":{},\"errors\":[{}],\"warnings\":[{}]}}}}",
                    report.valid, errors, warnings
                );
            } else if report.valid {
                println!("Project is valid");
                for warning in report.warnings {
                    println!("warning: {}: {}", warning.code, warning.message);
                }
            } else {
                println!("Project is invalid");
                for error in report.errors {
                    println!("error: {}: {}", error.code, error.message);
                }
                for warning in report.warnings {
                    println!("warning: {}: {}", warning.code, warning.message);
                }
            }

            if report.valid {
                0
            } else {
                3
            }
        }
        Err(error) => print_command_error("project", "validate", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  project create [--json] PARENT ID NAME TYPE");
    println!("  project target add-removable [--json] PATH NAME EXPORT_PATH");
    println!("  project list [--json] ROOT");
    println!("  project inspect [--json] PATH");
    println!("  project validate [--json] PATH");
}
