use std::env;
use std::path::Path;
use std::process;
use waystone_audio_metadata::list_recordings;
use waystone_cli_output::{escape_json, print_command_error};

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
        ["library", root] => library(root, json),
        ["help"] | ["--help"] | [] => {
            print_help();
            0
        }
        _ => {
            eprintln!("listen: usage error");
            print_help();
            2
        }
    }
}

fn library(root: &str, json: bool) -> i32 {
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
                println!("No playable recordings found");
            } else {
                for recording in recordings {
                    println!("{}\t{}", recording.id, recording.title);
                }
            }
            0
        }
        Err(error) => print_command_error("listen", "library", &error.to_string(), json),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  listen library [--json] ROOT");
}
