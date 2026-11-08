use super::InvocationObserver;
use std::fmt::Write;
use std::path::Path;

impl InvocationObserver {
    pub(in crate::diagnostics) fn hot_file_block(&self) -> String {
        let mut output = String::new();
        if let Some(files) = &self.hot_files {
            let entries = files
                .lock()
                .expect("hot file diagnostics mutex must not be poisoned")
                .entries
                .clone();
            for entry in entries.into_iter().flatten() {
                let _ = writeln!(
                    output,
                    "[hot] {}: {} {:.3}ms units={} (parallel; non-additive)",
                    entry.pass.label(),
                    escaped_path(&entry.path),
                    entry.duration.as_secs_f64() * 1000.0,
                    entry.units
                );
            }
        }
        output
    }
}

fn escaped_path(path: &Path) -> String {
    let mut output = String::new();
    for character in path.to_string_lossy().chars() {
        if character.is_control() || matches!(character, '\u{2028}' | '\u{2029}') {
            output.extend(character.escape_default());
        } else {
            output.push(character);
        }
    }
    output
}
