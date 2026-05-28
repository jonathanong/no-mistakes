use super::super::configured_plan_candidates::CoverageHints;
use super::super::diff_parser::DiffFile;
use super::hints_domains::{
    build_http_path_dependents, build_queue_job_dependents, build_route_path_dependents,
    removed_http_paths_per_file, removed_queue_jobs_per_file, removed_route_paths_per_file,
};
use super::TestFramework;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use no_mistakes::playwright::playwright_tests::{TestOccurrenceScope, TestPolicy};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

/// Fully-resolved selector configuration used to drive both the diff-side
/// scan and the test-side reverse-index extraction. Keeping the pieces
/// together makes their mismatch potential explicit: the diff scan can
/// match any attribute the project tracks (incl. `componentTestIds` values
/// and the HTML `id` attribute when enabled), but the test-side extractor
/// must use the narrower `testIds` configuration for the `getByTestId`
/// mapping so app-only attributes are not back-attributed onto test-ID
/// calls. `component_test_ids` is still threaded through to the test-side
/// CSS regex compilation so `page.locator('[data-qa=...]')` can be picked
/// up when `componentTestIds` configures `data-qa`.
struct SelectorSettings {
    diff_attributes: Vec<String>,
    test_id_attributes: Vec<String>,
    component_test_ids: std::collections::BTreeMap<String, String>,
    html_ids: bool,
}

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
    let settings = effective_selector_settings(config);
    if settings.diff_attributes.is_empty() {
        return CoverageHints::default();
    }

    let removed_selectors = removed_selectors_per_file(diff_files, &settings.diff_attributes);
    let removed_route_paths = removed_route_paths_per_file(diff_files);
    let removed_queue_jobs = removed_queue_jobs_per_file(diff_files);
    let removed_http_paths = removed_http_paths_per_file(diff_files);

    let any_removed = !removed_selectors.is_empty()
        || !removed_route_paths.is_empty()
        || !removed_queue_jobs.is_empty()
        || !removed_http_paths.is_empty();
    if !any_removed {
        return CoverageHints::default();
    }

    // Each reverse-index builder skips work when its corresponding removal
    // set is empty, so the per-test parse cost is paid only for domains
    // that actually have a candidate to match against.
    let selector_dependents = if removed_selectors.is_empty() {
        HashMap::new()
    } else {
        build_selector_dependents(all_tests, &settings)
    };
    let route_path_dependents = if removed_route_paths.is_empty() {
        HashMap::new()
    } else {
        build_route_path_dependents(all_tests)
    };
    let queue_job_dependents = if removed_queue_jobs.is_empty() {
        HashMap::new()
    } else {
        build_queue_job_dependents(all_tests)
    };
    let http_path_dependents = if removed_http_paths.is_empty() {
        HashMap::new()
    } else {
        build_http_path_dependents(all_tests)
    };

    CoverageHints {
        removed_selectors,
        selector_dependents,
        removed_route_paths,
        route_path_dependents,
        removed_queue_jobs,
        queue_job_dependents,
        removed_http_paths,
        http_path_dependents,
    }
}

fn effective_selector_settings(config: &NoMistakesConfig) -> SelectorSettings {
    let selectors = &config.tests.playwright.selectors;
    let mut test_id_attributes: Vec<String> = selectors
        .test_ids
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect();
    // Match the project's own selector-attribute default (data-testid AND
    // data-pw — see playwright::config::DEFAULT_SELECTOR_ATTRIBUTES) so a
    // diff that removes `data-pw="old"` populates the hint set even when
    // the v2 config leaves `tests.playwright.selectors.testIds` unset.
    // This trades a slightly broader `getByTestId(...)` mapping for
    // consistency with how the rest of the analyzer tracks selectors.
    if test_id_attributes.is_empty() {
        test_id_attributes.extend(["data-testid", "data-pw"].iter().map(|a| a.to_string()));
    }
    test_id_attributes.sort();
    test_id_attributes.dedup();

    // The diff-side scan covers everything the project tracks: the test-id
    // attributes above, plus the HTML attributes that `componentTestIds`
    // lowers to (the values, not the JSX prop names, since the diff sees
    // the rendered attribute) and the HTML `id` attribute when enabled.
    let mut diff_attributes = test_id_attributes.clone();
    for attribute in selectors.component_test_ids.values() {
        diff_attributes.push(attribute.clone());
    }
    if selectors.html_ids {
        diff_attributes.push("id".to_string());
    }
    diff_attributes.sort();
    diff_attributes.dedup();

    SelectorSettings {
        diff_attributes,
        test_id_attributes,
        component_test_ids: selectors.component_test_ids.clone(),
        html_ids: selectors.html_ids,
    }
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
    settings: &SelectorSettings,
) -> HashMap<(String, String), Vec<PathBuf>> {
    // Compile regexes with the project's real `html_ids` setting and its
    // `componentTestIds` map so the CSS `#id` shorthand and component-prop
    // CSS attribute selectors (e.g. `page.locator('[data-qa="x"]')`) are
    // picked up. Use the configured test-id attribute list for the
    // `getByTestId` mapping; passing the broader `diff_attributes` would
    // mis-attribute `getByTestId(...)` calls to app-only attributes.
    let regexes = no_mistakes::playwright::selectors::compile_selector_regexes_with_html_ids(
        &settings.test_id_attributes,
        &settings.component_test_ids,
        settings.html_ids,
    );
    let per_test: Vec<(PathBuf, Vec<(String, String)>)> = all_tests
        .par_iter()
        .map(|path| {
            (
                path.clone(),
                extract_test_selector_pairs(path, &settings.test_id_attributes, &regexes),
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
    test_id_attributes: &[String],
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
            program,
            source,
            regexes,
            test_id_attributes,
        )
    }) {
        Ok(occurrences) => occurrences,
        Err(_) => return Vec::new(),
    };
    // Mirror the policy/scope filtering that `analyze_test_occurrences`
    // applies before producing `Edge::Selector`, so a `test.skip(...)` body
    // or a teardown-hook locator does not surface a hint that the normal
    // analyzer would drop.
    let policy = TestPolicy::default();
    let mut pairs: Vec<(String, String)> = Vec::new();
    for occ in occurrences {
        if !policy.allows(occ.status) {
            continue;
        }
        if occ.scope == TestOccurrenceScope::TeardownHook {
            continue;
        }
        if let Some(value) = occ.value.exact_value() {
            pairs.push((occ.value.attribute.clone(), value.to_string()));
        }
    }
    pairs.sort();
    pairs.dedup();
    pairs
}
