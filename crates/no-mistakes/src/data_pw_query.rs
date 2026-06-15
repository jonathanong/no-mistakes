//! `data-pw` query: find every `attribute="value"` selector usage (for example
//! `data-pw="search-bar"`) across component source files and test files.
//!
//! The attribute names are config-driven (`tests.playwright.selectors.testIds`,
//! e.g. `["data-testid", "data-pw"]`) and may be overridden per-invocation with
//! `--attribute`. Source vs. test classification is by file membership: a file
//! that matches the configured Playwright `testInclude` globs is a test file,
//! everything else scanned is source.
//!
//! Matching reuses the anchored attribute-value regex from the Playwright
//! selector scanner ([`compile_selector_attribute_value_regex`]), so it catches
//! JSX attributes (`<div data-pw="x">`) and CSS attribute selectors
//! (`page.locator('[data-pw="x"]')`) alike, and skips dynamic values
//! (`data-pw={x}`). Implicit references such as `getByTestId('x')` that do not
//! spell out `attribute="value"` are intentionally not matched; see the docs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{load_v2_config, ConfigView};
use crate::playwright::selectors::compile_selector_attribute_value_regex;

const SOURCE_EXTENSIONS: &[&str] = &["tsx", "ts", "jsx", "js", "mts", "cts", "mjs", "cjs"];

/// Which sections to include in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataPwInclude {
    pub source: bool,
    pub test: bool,
}

impl Default for DataPwInclude {
    fn default() -> Self {
        Self {
            source: true,
            test: true,
        }
    }
}

impl DataPwInclude {
    /// Parse a comma-separated `--include` spec (subset of `source,test`).
    /// `None` includes both sections.
    pub fn parse(spec: Option<&str>) -> Result<Self> {
        let Some(spec) = spec else {
            return Ok(Self::default());
        };
        let mut include = Self {
            source: false,
            test: false,
        };
        for raw in spec.split(',') {
            match raw.trim() {
                "" => continue,
                "source" => include.source = true,
                "test" => include.test = true,
                other => bail!("unknown --include section: {other} (expected source,test)"),
            }
        }
        if !include.source && !include.test {
            bail!("--include must name at least one of: source,test");
        }
        Ok(include)
    }
}

/// A single `attribute="value"` occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DataPwHit {
    pub file: String,
    pub line: usize,
    pub attribute: String,
}

/// The full `data-pw <value>` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DataPwReport {
    pub value: String,
    pub attributes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<DataPwHit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<Vec<DataPwHit>>,
}

impl DataPwReport {
    /// Deduplicated, sorted union of all matched file paths, for `--format paths`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: BTreeSet<String> = BTreeSet::new();
        for section in [&self.source, &self.test].into_iter().flatten() {
            for hit in section {
                paths.insert(hit.file.clone());
            }
        }
        paths.into_iter().collect()
    }
}

/// Run the `data-pw` query.
///
/// * `attribute_override` — when non-empty, scan only these attributes instead
///   of the configured `testIds`.
/// * `scan_override` — when non-empty, restrict source scanning to these path
///   prefixes instead of the configured `selectorRoots`.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    value: &str,
    attribute_override: &[String],
    scan_override: &[String],
    include: &DataPwInclude,
) -> Result<DataPwReport> {
    let config = load_v2_config(root, config_path)?;
    let view = ConfigView::new(&config);

    let attributes: Vec<String> = if attribute_override.is_empty() {
        view.test_id_attributes().to_vec()
    } else {
        attribute_override.to_vec()
    };
    let Some(regex) = compile_selector_attribute_value_regex(&attributes) else {
        bail!(
            "no selector attributes configured; set tests.playwright.selectors.testIds \
             or pass --attribute"
        );
    };

    let roots: Vec<String> = if scan_override.is_empty() {
        view.selector_roots().to_vec()
    } else {
        scan_override.to_vec()
    };
    let normalized_roots: Vec<String> = roots
        .iter()
        .map(|root| root.trim_matches('/').trim_start_matches("./").to_string())
        .filter(|root| !root.is_empty())
        .collect();

    let test_globs = build_globset(&config.tests.playwright.test_include)?;
    let exclude_globs = build_globset(view.selector_exclude())?;

    let files = discover_files(root);
    let hits: Vec<(FileKind, DataPwHit)> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(root, path);
            scan_file(
                path,
                &rel,
                value,
                &regex,
                &normalized_roots,
                &test_globs,
                &exclude_globs,
            )
        })
        .collect();

    let mut source: Vec<DataPwHit> = Vec::new();
    let mut test: Vec<DataPwHit> = Vec::new();
    for (kind, hit) in hits {
        match kind {
            FileKind::Source => source.push(hit),
            FileKind::Test => test.push(hit),
        }
    }
    source.sort();
    source.dedup();
    test.sort();
    test.dedup();

    Ok(DataPwReport {
        value: value.to_string(),
        attributes,
        source: include.source.then_some(source),
        test: include.test.then_some(test),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Source,
    Test,
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

fn discover_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !(entry.file_type().is_dir() && is_skip_dir(entry.path())));
    for entry in walker.filter_map(|entry| entry.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if SOURCE_EXTENSIONS.contains(&ext) {
            files.push(path.to_path_buf());
        }
    }
    files
}

fn is_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with('.')
                || matches!(
                    name,
                    "node_modules" | "target" | "dist" | "build" | "coverage"
                )
        })
}

#[allow(clippy::too_many_arguments)]
fn scan_file(
    path: &Path,
    rel: &str,
    value: &str,
    regex: &regex::Regex,
    roots: &[String],
    test_globs: &GlobSet,
    exclude_globs: &GlobSet,
) -> Vec<(FileKind, DataPwHit)> {
    if exclude_globs.is_match(rel) {
        return Vec::new();
    }
    let is_test = test_globs.is_match(rel);
    let in_source_root = roots.is_empty() || roots.iter().any(|root| path_in_root(rel, root));
    if !is_test && !in_source_root {
        return Vec::new();
    }
    let kind = if is_test {
        FileKind::Test
    } else {
        FileKind::Source
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for (index, line) in source.lines().enumerate() {
        for caps in regex.captures_iter(line) {
            let attribute = &caps["attr"];
            let matched = caps
                .name("dq")
                .or_else(|| caps.name("sq"))
                .map(|m| m.as_str())
                .unwrap_or("");
            if matched == value {
                hits.push((
                    kind,
                    DataPwHit {
                        file: rel.to_string(),
                        line: index + 1,
                        attribute: attribute.to_string(),
                    },
                ));
            }
        }
    }
    hits
}

/// Whether `rel` lives under directory prefix `root` (e.g. `app` matches
/// `app/page.tsx` but not `apply.ts`).
fn path_in_root(rel: &str, root: &str) -> bool {
    rel == root || rel.starts_with(&format!("{root}/"))
}

#[cfg(test)]
mod tests;
