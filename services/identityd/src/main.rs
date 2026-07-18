fn main() {
    if let Err(error) = dbus::run() {
        eprintln!("waystone-identityd: {error}");
        std::process::exit(1);
    }
}

mod dbus;
