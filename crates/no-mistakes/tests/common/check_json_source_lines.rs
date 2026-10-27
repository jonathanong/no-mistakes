//! Adds the trimmed source line to each normalized finding.
//!
//! A snapshot that stores only `file` and `line` cannot show a rule pointing
//! at the wrong line. The text is read from the saved fixture (or the checkout
//! for `<repo>/` config findings) after path normalization, folded to LF, and
//! trimmed, so it is identical on every operating system. Paths that resolve
//! outside the fixture root (or checkout) get no `sourceLine`.

use serde_json::{Map, Value};
use std::path::Path;

pub fn add_source_lines(value: &mut Value, root: &Path, repo: &Path) {
    match value {
        Value::Array(items) => {
            for item in items {
                add_source_lines(item, root, repo);
            }
        }
        Value::Object(map) => {
            if let Some(text) = finding_source(map, root, repo) {
                map.insert("sourceLine".to_string(), Value::String(text));
            }
            for child in map.values_mut() {
                add_source_lines(child, root, repo);
            }
        }
        _ => {}
    }
}

fn finding_source(map: &Map<String, Value>, root: &Path, repo: &Path) -> Option<String> {
    let file = map.get("file")?.as_str()?;
    let line = map.get("line")?.as_u64()?;
    if Path::new(file).is_absolute() {
        return None;
    }
    let (base, rest) = match file.strip_prefix("<repo>/") {
        Some(rest) => (repo, rest),
        None => (root, file),
    };
    // A `../` segment (for example a stale baseline entry for a project
    // outside the fixture) must not read files outside the tree it names.
    let base = base.canonicalize().ok()?;
    let path = base.join(rest).canonicalize().ok()?;
    if !path.starts_with(&base) {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    source_line(&text, line)
}

pub fn source_line(text: &str, line: u64) -> Option<String> {
    let index = usize::try_from(line.checked_sub(1)?).ok()?;
    // CRLF, CR, LF, U+2028 and U+2029 each end a line, matching how the
    // TS/JS parser numbers lines, so a CR-only fixture still lines up.
    let folded = text.replace("\r\n", "\n");
    folded
        .split(['\n', '\r', '\u{2028}', '\u{2029}'])
        .nth(index)
        .map(|row| row.trim().to_string())
}
