use super::{Options, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn stem_and_dir(rel: &str, test_ext: &str) -> (String, String) {
    let stem = rel.strip_suffix(test_ext).unwrap_or(rel);
    match stem.rfind('/') {
        Some(i) => (stem[..i].to_string(), stem[i + 1..].to_string()),
        None => (String::new(), stem.to_string()),
    }
}

pub(crate) fn source_candidates(dir: &str, stem: &str, test_ext: &str) -> Vec<String> {
    let p = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };
    let src_ext = source_extension_family(test_ext);
    match src_ext {
        // Module-specific: only look for the exact extension
        "mts" | "cts" | "mjs" | "cjs" => vec![
            format!("{p}{stem}.{src_ext}"),
            format!("{p}index.{src_ext}"),
        ],
        "js" | "jsx" => vec![
            format!("{p}{stem}.js"),
            format!("{p}{stem}.jsx"),
            format!("{p}index.js"),
            format!("{p}index.jsx"),
        ],
        _ => vec![
            format!("{p}{stem}.ts"),
            format!("{p}{stem}.tsx"),
            format!("{p}index.ts"),
            format!("{p}index.tsx"),
        ],
    }
}

fn source_extension_family(test_ext: &str) -> &str {
    test_ext.rsplit('.').next().unwrap_or("ts")
}

fn source_extensions_for_test_ext(test_ext: &str) -> Vec<&'static str> {
    match source_extension_family(test_ext) {
        "mts" => vec!["mts"],
        "cts" => vec!["cts"],
        "mjs" => vec!["mjs"],
        "cjs" => vec!["cjs"],
        "js" | "jsx" => vec!["js", "jsx"],
        _ => vec!["ts", "tsx"],
    }
}

pub(super) fn check_source_to_test(
    files: &[PathBuf],
    root: &Path,
    opts: &Options,
    exts: &[&str],
    rel_set: &HashSet<String>,
    tdir: &str,
) -> Vec<RuleFinding> {
    let sep = format!("/{tdir}/");
    let pre = format!("{tdir}/");
    let src_exts: HashSet<&str> = exts
        .iter()
        .flat_map(|e| source_extensions_for_test_ext(e))
        .collect();

    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .filter_map(|path| {
            let rel = relative_slash_path(root, path);

            if !opts.scopes.is_empty()
                && !opts
                    .scopes
                    .iter()
                    .any(|s| rel == *s || rel.starts_with(&format!("{s}/")))
            {
                return None;
            }

            if exts.iter().any(|&e| rel.ends_with(e)) || rel.contains(&sep) || rel.starts_with(&pre)
            {
                return None;
            }
            if is_declaration_file(&rel) {
                return None;
            }

            // Only process source extensions derived from test extensions
            let ext = path.extension().and_then(|e| e.to_str())?;
            if !src_exts.contains(ext) {
                return None;
            }

            let dot_ext = format!(".{ext}");
            let (dir, base) = stem_and_dir(&rel, &dot_ext);
            let p = if dir.is_empty() {
                String::new()
            } else {
                format!("{dir}/")
            };
            let tdir_p = if dir.is_empty() {
                format!("{tdir}/")
            } else {
                format!("{dir}/{tdir}/")
            };

            let found = exts.iter().any(|&test_ext| {
                if rel_set.contains(&format!("{p}{base}{test_ext}"))
                    || rel_set.contains(&format!("{tdir_p}{base}{test_ext}"))
                {
                    return true;
                }
                opts.stem_suffixes_to_strip.iter().any(|suffix| {
                    rel_set.contains(&format!("{p}{base}{suffix}{test_ext}"))
                        || rel_set.contains(&format!("{tdir_p}{base}{suffix}{test_ext}"))
                })
            });

            if found {
                None
            } else {
                Some(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.clone(),
                    line: 1,
                    message: format!("{rel}: no corresponding test file found"),
                    import: None,
                    target: None,
                })
            }
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    findings
}

fn is_declaration_file(rel: &str) -> bool {
    rel.ends_with(".d.ts") || rel.ends_with(".d.mts") || rel.ends_with(".d.cts")
}
