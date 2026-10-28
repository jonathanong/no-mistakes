//! Best-effort 1-based source line of the key or declaration a set was read
//! from, so findings point at the compared YAML/JSON key or TS declaration
//! instead of line 1. An unresolved key falls back to the nearest resolved
//! ancestor, then line 1; it is never matched at the wrong depth.

mod declaration;
mod entries;
mod json_walk;
mod yaml_walk;

use super::comments::{strip_comments, strip_sql_comments};
use super::SetSpec;
use declaration::{blank_strings, declaration_line};

pub(super) fn set_line(spec: &SetSpec, source: &str) -> usize {
    match spec.kind.as_str() {
        "yaml-sequence" => {
            let path = spec.key.split('.').map(String::from).collect::<Vec<_>>();
            key_line(source, &path)
        }
        "yaml-string-selector" => {
            super::yaml::selector_key_path(&spec.key).map_or(1, |path| key_line(source, &path))
        }
        "ts-string-union"
        | "ts-const-object-keys"
        | "ts-const-object-property"
        | "ts-array-literal"
        | "ts-const-array-property" => declaration_line(
            &blank_strings(&strip_comments(source)),
            r"\b(?:type|const|let|var|enum|interface|function|class)\s+",
            &spec.target,
        ),
        "sql-enum" => declaration_line(
            &strip_sql_comments(source),
            r"(?i)\bCREATE\s+TYPE\s+",
            &spec.target,
        ),
        _ => 1,
    }
}

/// JSON documents (opening brace or bracket) use the token walker. When it
/// stops on text it cannot parse, the YAML line scan also runs and the deeper
/// (later) line wins, so this is never less precise than the line scan alone.
fn key_line(source: &str, path: &[String]) -> usize {
    if source.trim_start().starts_with(['{', '[']) {
        let (line, parsed) = json_walk::json_key_line(source, path);
        if parsed {
            return line;
        }
        return line.max(yaml_walk::yaml_key_line(source, path));
    }
    yaml_walk::yaml_key_line(source, path)
}
