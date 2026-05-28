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
    let selectors = &config.tests.playwright.selectors;
    let mut attributes: Vec<String> = selectors
        .test_ids
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect();
    if attributes.is_empty() {
        attributes.extend(["data-testid", "data-pw"].iter().map(|a| a.to_string()));
    }
    // `componentTestIds` maps a JSX prop name (e.g. `dataPw`) to the HTML
    // attribute it lowers to (e.g. `data-pw`); the latter is what shows up on
    // a `-` line in a diff, so add it (the prop name lives in JSX, not the
    // rendered HTML, so do not include it here).
    for attribute in selectors.component_test_ids.values() {
        attributes.push(attribute.clone());
    }
    if selectors.html_ids {
        attributes.push("id".to_string());
    }
    attributes.sort();
    attributes.dedup();
    attributes
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
    let regexes = no_mistakes::playwright::selectors::compile_selector_regexes(
        attributes,
        &std::collections::BTreeMap::new(),
    );
    let per_test: Vec<(PathBuf, Vec<(String, String)>)> = all_tests
        .par_iter()
        .map(|path| {
            (
                path.clone(),
                extract_test_selector_pairs(path, attributes, &regexes),
            )
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

fn extract_test_selector_pairs(
    path: &std::path::Path,
    attributes: &[String],
    regexes: &no_mistakes::playwright::selectors::SelectorRegexes,
) -> Vec<(String, String)> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    // Use the real file path so `SourceType::from_path` picks up `.tsx`/`.jsx`
    // and JSX-bearing specs parse cleanly. Treat parse failures as a quiet
    // skip — the goal here is best-effort augmentation, not authoritative
    // analysis, and the playwright analyzer will surface the parse error.
    let occurrences = match no_mistakes::ast::with_program(path, &source, |program, source| {
        no_mistakes::playwright::selectors::extract_playwright_selector_occurrences_from_program(
            program, source, regexes, attributes,
        )
    }) {
        Ok(occurrences) => occurrences,
        Err(_) => return Vec::new(),
    };
    let mut pairs: Vec<(String, String)> = Vec::new();
    for occ in occurrences {
        if let Some(value) = occ.value.exact_value() {
            pairs.push((occ.value.attribute.clone(), value.to_string()));
        }
    }
    pairs.sort();
    pairs.dedup();
    pairs
}
