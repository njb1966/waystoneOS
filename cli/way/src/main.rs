use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    process::exit(run(&args));
}

fn run(args: &[String]) -> i32 {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["help"] | ["--help"] | [] => {
            print_help();
            0
        }
        _ => {
            eprintln!("way: usage error");
            print_help();
            2
        }
    }
}

fn print_help() {
    println!("WaystoneOS command entrypoint");
    println!();
    println!("Core commands:");
    println!("  project   Create, inspect, list, and validate .wayproject directories");
    println!("  publish   Preview publication plans without remote mutation");
    println!("  host      Inspect and validate trusted host metadata");
    println!("  identity  Inspect and validate publishing identity metadata");
    println!("  record    Inspect and validate recording metadata");
    println!("  listen    List playable recordings from a project");
    println!();
    println!("Service binaries:");
    println!("  waystone-projectd    Project D-Bus service");
    println!("  waystone-hostd       Host D-Bus service");
    println!("  waystone-identityd   Identity D-Bus service");
    println!("  waystone-audiod      Audio service placeholder");
}
