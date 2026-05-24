use super::{Options, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
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
    let src_ext = test_ext.rsplit('.').next().unwrap_or("ts");
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
    let src_exts: HashSet<&str> = exts.iter().filter_map(|e| e.rsplit('.').next()).collect();

    let mut findings = Vec::new();

    for path in files {
        let rel = relative_slash_path(root, path);

        if !opts.scopes.is_empty()
            && !opts
                .scopes
                .iter()
                .any(|s| rel == *s || rel.starts_with(&format!("{s}/")))
        {
            continue;
        }

        if exts.iter().any(|&e| rel.ends_with(e)) || rel.contains(&sep) || rel.starts_with(&pre) {
            continue;
        }

        // Only process source extensions derived from test extensions
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };
        if !src_exts.contains(ext) {
            continue;
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
            rel_set.contains(&format!("{p}{base}{test_ext}"))
                || rel_set.contains(&format!("{tdir_p}{base}{test_ext}"))
        });

        if !found {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: no corresponding test file found"),
                import: None,
                target: None,
            });
        }
    }

    findings
}
