pub fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

pub fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn json_error(operation: &str, message: &str) -> String {
    format!(
        "{{\"status\":\"error\",\"schema\":1,\"error\":{{\"code\":\"{}_failed\",\"message\":\"{}\"}}}}",
        escape_json(operation),
        escape_json(message)
    )
}

pub fn print_command_error(command: &str, operation: &str, message: &str, json: bool) -> i32 {
    if json {
        println!("{}", json_error(operation, message));
    } else {
        eprintln!("{command}: {operation} failed: {message}");
    }
    1
}

#[cfg(test)]
mod tests {
    use super::{escape_json, json_error, json_optional_string, json_string_array};

    #[test]
    fn escapes_json_control_characters_used_by_commands() {
        assert_eq!(escape_json("a\\b\"c\nd"), "a\\\\b\\\"c\\nd");
    }

    #[test]
    fn renders_optional_string_values() {
        assert_eq!(json_optional_string(Some("capsule")), "\"capsule\"");
        assert_eq!(json_optional_string(None), "null");
    }

    #[test]
    fn renders_string_arrays() {
        let values = vec!["one".to_string(), "two".to_string()];
        assert_eq!(json_string_array(&values), "\"one\",\"two\"");
    }

    #[test]
    fn renders_standard_error_envelope() {
        assert_eq!(
            json_error("dry_run", "missing target"),
            "{\"status\":\"error\",\"schema\":1,\"error\":{\"code\":\"dry_run_failed\",\"message\":\"missing target\"}}"
        );
    }
}
