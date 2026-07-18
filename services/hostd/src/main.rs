fn main() {
    if let Err(error) = dbus::run() {
        eprintln!("waystone-hostd: {error}");
        std::process::exit(1);
    }
}

mod dbus;
