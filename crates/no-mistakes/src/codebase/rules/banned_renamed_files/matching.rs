use super::{BannedBasename, Options, RULE_ID};
use anyhow::{Context, Result};
use regex::Regex;

/// A banned basename paired with its precompiled regex (when `pattern` is set).
/// Compiling once per rule application avoids recompiling across every scanned file
/// and lets invalid-regex errors surface through the rule's `Result` chain.
pub(crate) struct CompiledBasename<'a> {
    pub(crate) def: &'a BannedBasename,
    pub(crate) regex: Option<Regex>,
}

pub(crate) fn compile_basenames(opts: &Options) -> Result<Vec<CompiledBasename<'_>>> {
    opts.banned_basenames
        .iter()
        .map(|def| {
            let regex = match &def.pattern {
                Some(pattern) => Some(Regex::new(pattern).with_context(|| {
                    format!(
                        "{RULE_ID}: invalid `pattern` regex {pattern:?} for basename {:?}",
                        def.name
                    )
                })?),
                None => None,
            };
            Ok(CompiledBasename { def, regex })
        })
        .collect()
}

/// Does `file_name` match this banned entry? Precedence: explicit `pattern` regex wins;
/// otherwise `matchCompoundExtensions` allows multi-part suffixes; otherwise exact stem.
pub(crate) fn matches(file_name: &str, compiled: &CompiledBasename, extensions: &[String]) -> bool {
    if let Some(regex) = &compiled.regex {
        return regex.is_match(file_name);
    }

    let (stem, ext) = split_stem_ext(file_name);
    if !extensions.iter().any(|e| e.strip_prefix('.') == Some(ext)) {
        return false;
    }

    let name = compiled.def.name.as_str();
    if compiled.def.match_compound_extensions {
        // `<name>` followed by a `.` and at least one more segment.
        stem == name
            || stem
                .strip_prefix(name)
                .is_some_and(|rest| rest.starts_with('.'))
    } else {
        stem == name
    }
}

fn split_stem_ext(filename: &str) -> (&str, &str) {
    match filename.rfind('.') {
        Some(i) if i > 0 => (&filename[..i], &filename[i + 1..]),
        _ => (filename, ""),
    }
}
