//! Shell command for one SwiftPM test target.

use crate::impacted_checks::shell_quote;

pub(super) fn test_command(package: &str, target: &str, base: Option<&[String]>) -> String {
    // `swift test --filter` matches a regex against `<target>.<case>`; anchor to
    // the target prefix and escape it so one target name cannot match another.
    // Single-quote it so the shell preserves the regex backslashes verbatim.
    let filter = shell_single_quote(&format!("^{}\\.", regex_escape(target)));
    // `None` stays the historical `swift test` text. A declared argv is quoted
    // the same way as `tests plan --format commands`.
    let prefix = match base {
        None => "swift test".to_string(),
        Some(argv) => argv
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" "),
    };
    if package.is_empty() || package == "." {
        format!("{prefix} --filter {filter}")
    } else {
        format!(
            "{prefix} --package-path {} --filter {filter}",
            shell_single_quote(package)
        )
    }
}

fn regex_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if "\\.+*?()|[]{}^$".contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Wrap a value in single quotes for POSIX shells, escaping embedded quotes so a
/// package path like `bob's app` stays a single argument.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
