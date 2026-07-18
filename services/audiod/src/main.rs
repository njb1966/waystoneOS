fn main() {
    if let Err(error) = dbus::run() {
        eprintln!("waystone-audiod: {error}");
        std::process::exit(1);
    }
}

mod dbus;
