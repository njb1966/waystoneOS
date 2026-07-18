mod dbus;

fn main() {
    if let Err(error) = dbus::run() {
        eprintln!("waystone-projectd: {error}");
        std::process::exit(1);
    }
}
