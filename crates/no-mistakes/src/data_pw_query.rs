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

use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{load_v2_config_from_visible, ConfigView};
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
    // VisiblePathSnapshot returns lexically normalized paths. Normalize the
    // matching boundary once too so roots containing `.`/`..` still produce
    // relative report paths and pass the visibility filter.
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    crate::invocation::check_timeout()?;
    let config = load_v2_config_from_visible(&root, config_path, &visible_paths)?;
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
    // A `.`/`./`/empty root anywhere means "scan the whole repo": collapse to the
    // all-roots (empty) case rather than a literal `.` prefix that matches nothing.
    let normalize_root = |root: &String| {
        root.trim()
            .trim_matches('/')
            .trim_start_matches("./")
            .to_string()
    };
    let has_all_root = roots
        .iter()
        .any(|root| matches!(normalize_root(root).as_str(), "" | "."));
    let normalized_roots: Vec<String> = if has_all_root {
        Vec::new()
    } else {
        roots.iter().map(normalize_root).collect()
    };

    let playwright = &config.tests.playwright;
    let selector_include = if playwright.selector_include.is_empty() {
        None
    } else {
        Some(build_globset(&playwright.selector_include)?)
    };
    let scan = ScanConfig {
        value,
        regex: &regex,
        roots: &normalized_roots,
        test_globs: &build_globset(&playwright.test_include)?,
        test_exclude_globs: &build_globset(&playwright.test_exclude)?,
        selector_include_globs: selector_include.as_ref(),
        exclude_globs: &build_globset(view.selector_exclude())?,
    };

    let files = discover_files_from_visible_paths(&root, &visible_paths, view.skip_directories());
    let hits: Vec<(FileKind, DataPwHit)> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(&root, path);
            scan_file(path, &rel, &scan)
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

    crate::invocation::check_timeout()?;
    Ok(DataPwReport {
        value: value.to_string(),
        attributes,
        source: include.source.then_some(source),
        test: include.test.then_some(test),
    })
}

include!("data_pw_query/scan.rs");

#[cfg(test)]
mod tests;
