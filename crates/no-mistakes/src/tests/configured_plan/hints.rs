use super::super::configured_plan_candidates::CoverageHints;
use super::super::diff_parser::DiffFile;
use super::TestFramework;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

/// Build the per-run coverage hints used by `graph_candidates` to surface
/// tests at risk when a unified diff removes an identifier. Currently
/// limited to playwright selector renames; returns an empty `CoverageHints`
/// for other frameworks or when no removed selectors are detected.
pub(super) fn build_coverage_hints(
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

    let removed_selectors = removed_selectors_per_file(diff_files, &attributes);
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
    diff_files: &[DiffFile],
    attributes: &[String],
) -> BTreeMap<PathBuf, Vec<(String, String)>> {
    let mut out: BTreeMap<PathBuf, Vec<(String, String)>> = BTreeMap::new();
    let Some(re) =
        no_mistakes::playwright::selectors::compile_selector_attribute_value_regex(attributes)
    else {
        return out;
    };
    let per_file: Vec<(PathBuf, Vec<(String, String)>)> = diff_files
        .par_iter()
        .filter_map(|df| truly_removed_for_file(&re, df))
        .collect();
    // `collect_changed_files` already absolutizes and normalizes each
    // `DiffFile::path`, so it is used directly as the map key. Sorting via
    // BTreeMap and sorting/dedup of the merged pair list keep the output
    // deterministic across parallel runs.
    for (key, pairs) in per_file {
        out.entry(key).or_default().extend(pairs);
    }
    for pairs in out.values_mut() {
        pairs.sort();
        pairs.dedup();
    }
    out
}

fn truly_removed_for_file(
    re: &regex::Regex,
    df: &DiffFile,
) -> Option<(PathBuf, Vec<(String, String)>)> {
    if df.removed_lines.is_empty() {
        return None;
    }
    let removed = no_mistakes::playwright::selectors::scan_selector_attribute_values_with_regex(
        re,
        &df.removed_lines,
    );
    if removed.is_empty() {
        return None;
    }
    let added: HashSet<(String, String)> =
        no_mistakes::playwright::selectors::scan_selector_attribute_values_with_regex(
            re,
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
        return None;
    }
    Some((df.path.clone(), truly_removed))
}

fn build_selector_dependents(
    all_tests: &[PathBuf],
    attributes: &[String],
) -> HashMap<(String, String), Vec<PathBuf>> {
    let per_test: Vec<(PathBuf, Vec<(String, String)>)> = all_tests
        .par_iter()
        .map(|path| {
            let Ok(source) = std::fs::read_to_string(path) else {
                return (path.clone(), Vec::new());
            };
            let selectors = no_mistakes::playwright::selectors::extract_playwright_selectors(
                &source, attributes, attributes,
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
