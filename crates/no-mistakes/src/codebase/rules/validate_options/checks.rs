use crate::config::source_locations::{ConfigPath, Segment};
use crate::config::v2::schema::OptionLocation;
use anyhow::Result;
use globset::Glob;
use std::fmt::Display;

pub(in crate::codebase::rules) fn option(key: &str) -> ConfigPath {
    ConfigPath::default().key("options").key(key)
}

pub(in crate::codebase::rules) fn invalid(
    rule: &str,
    path: ConfigPath,
    detail: impl Display,
) -> anyhow::Error {
    let mut field = String::new();
    for segment in path.as_slice() {
        match segment {
            Segment::Key(name) => {
                if !field.is_empty() {
                    field.push('.');
                }
                field.push_str(name);
            }
            Segment::Index(index) => field.push_str(&format!("[{index}]")),
        }
    }
    OptionLocation::semantic(
        path,
        format!("invalid options for rule `{rule}` at {field}: {detail}"),
    )
}

pub(in crate::codebase::rules) fn nonblank(
    rule: &str,
    path: ConfigPath,
    value: &str,
) -> Result<()> {
    if value.trim().is_empty() {
        return Err(invalid(rule, path, "must be nonblank"));
    }
    Ok(())
}

pub(in crate::codebase::rules) fn includes(rule: &str, patterns: &[String]) -> Result<()> {
    globs(rule, "include", patterns, |pattern| {
        pattern.trim_start_matches("./")
    })
}

pub(super) fn globs(
    rule: &str,
    key: &str,
    patterns: &[String],
    normalize: impl Fn(&str) -> &str,
) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        Glob::new(normalize(pattern))
            .map_err(|error| invalid(rule, option(key).index(index), error))?;
    }
    Ok(())
}
