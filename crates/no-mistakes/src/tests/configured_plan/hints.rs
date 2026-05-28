use super::super::configured_plan_candidates::CoverageHints;
use super::super::diff_parser::DiffFile;
use super::TestFramework;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Build the per-run coverage hints used by `graph_candidates` to surface
/// tests at risk when a unified diff removes an identifier. Currently
/// limited to playwright selector renames; returns an empty `CoverageHints`
/// for other frameworks or when no removed selectors are detected.
pub(super) fn build_coverage_hints(
    root: &Path,
    config: &NoMistakesConfig,
    framework: TestFramework,
    diff_files: &[DiffFile],
    all_tests: &[PathBuf],
) -> CoverageHints {
    if framework != TestFramework::Playwright {
        return CoverageHints::default();
    }
    let attributes = effective_selector_attributes(config);
    if attributes.is_empty() {
        return CoverageHints::default();
    }

    let removed_selectors = removed_selectors_per_file(root, diff_files, &attributes);
    if removed_selectors.is_empty() {
        return CoverageHints::default();
    }

    let selector_dependents = build_selector_dependents(all_tests, &attributes);

    CoverageHints {
        removed_selectors,
        selector_dependents,
    }
}

fn effective_selector_attributes(config: &NoMistakesConfig) -> Vec<String> {
    let configured: Vec<String> = config
        .tests
        .playwright
        .selectors
        .test_ids
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect();
    if !configured.is_empty() {
        return configured;
    }
    vec!["data-testid".to_string(), "data-pw".to_string()]
}

fn removed_selectors_per_file(
    root: &Path,
    diff_files: &[DiffFile],
    attributes: &[String],
) -> BTreeMap<PathBuf, Vec<(String, String)>> {
    let mut out: BTreeMap<PathBuf, Vec<(String, String)>> = BTreeMap::new();
    for df in diff_files {
        if df.removed_lines.is_empty() {
            continue;
        }
        let removed = no_mistakes::playwright::selectors::scan_selector_attribute_values(
            attributes,
            &df.removed_lines,
        );
        if removed.is_empty() {
            continue;
        }
        let added: HashSet<(String, String)> =
            no_mistakes::playwright::selectors::scan_selector_attribute_values(
                attributes,
                &df.added_lines,
            )
            .into_iter()
            .collect();
        let mut truly_removed: Vec<(String, String)> = Vec::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for pair in removed {
            if added.contains(&pair) {
                continue;
            }
            if seen.insert(pair.clone()) {
                truly_removed.push(pair);
            }
        }
        if truly_removed.is_empty() {
            continue;
        }
        let absolute = if df.path.is_absolute() {
            df.path.clone()
        } else {
            root.join(&df.path)
        };
        let key = no_mistakes::codebase::ts_resolver::normalize_path(&absolute);
        out.entry(key).or_default().extend(truly_removed);
    }
    out
}

fn build_selector_dependents(
    all_tests: &[PathBuf],
    attributes: &[String],
) -> HashMap<(String, String), Vec<PathBuf>> {
    let test_id_attributes: Vec<String> = attributes.to_vec();
    let per_test: Vec<(PathBuf, Vec<(String, String)>)> = all_tests
        .par_iter()
        .map(|path| {
            let Ok(source) = std::fs::read_to_string(path) else {
                return (path.clone(), Vec::new());
            };
            let selectors = no_mistakes::playwright::selectors::extract_playwright_selectors(
                &source,
                attributes,
                &test_id_attributes,
            );
            let mut pairs: Vec<(String, String)> = Vec::new();
            for sel in selectors {
                if let Some(value) = sel.exact_value() {
                    pairs.push((sel.attribute.clone(), value.to_string()));
                }
            }
            pairs.sort();
            pairs.dedup();
            (path.clone(), pairs)
        })
        .collect();
    let mut out: HashMap<(String, String), Vec<PathBuf>> = HashMap::new();
    for (test, pairs) in per_test {
        for pair in pairs {
            out.entry(pair).or_default().push(test.clone());
        }
    }
    for tests in out.values_mut() {
        tests.sort();
        tests.dedup();
    }
    out
}
