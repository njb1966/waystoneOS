use std::process::Command;

#[test]
fn help_lists_current_core_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_way"))
        .arg("--help")
        .output()
        .expect("way command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WaystoneOS command entrypoint"));
    assert!(stdout.contains("project"));
    assert!(stdout.contains("publish"));
    assert!(stdout.contains("host"));
    assert!(stdout.contains("identity"));
    assert!(stdout.contains("record"));
    assert!(stdout.contains("listen"));
}
