use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

const RULE: &str = "playwright-unique-test-ids";

fn effort_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/playwright-unique-test-ids/fixture/effort-low"),
    )
}

fn corpus_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/playwright-unique-test-ids/fixture"),
    )
}

fn run(
    root: &Path,
    config: Option<&str>,
    effort: Option<Effort>,
) -> crate::check_runner::CheckResults {
    let config_path = config.map(|name| root.join(name));
    run_all_with_options(
        root.to_path_buf(),
        config_path,
        None,
        false,
        false,
        AnalysisOptions {
            effort,
            include_sql_verdicts: false,
        },
    )
    .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
}

fn read(root: &Path, file: &str) -> String {
    std::fs::read_to_string(root.join(file)).unwrap()
}

fn marked_lines(source: &str, marker: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
        .collect()
}

fn lines_where(
    rows: &[no_mistakes::codebase::rules::RuleFinding],
    file: &str,
    message: &str,
) -> Vec<usize> {
    let mut lines = rows
        .iter()
        .filter(|row| row.rule == RULE && row.file == file && row.message.contains(message))
        .map(|row| row.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

fn shape_lines(rows: &[no_mistakes::codebase::rules::RuleFinding], file: &str) -> Vec<usize> {
    lines_where(rows, file, "unanalyzable at effort low")
}

fn duplicate_targets(rows: &[no_mistakes::codebase::rules::RuleFinding]) -> Vec<&str> {
    let mut targets = rows
        .iter()
        .filter(|row| row.rule == RULE && row.message.contains("duplicated"))
        .filter_map(|row| row.target.as_deref())
        .collect::<Vec<_>>();
    targets.sort_unstable();
    targets
}

#[test]
fn prepared_max_matches_explicit_max_on_the_existing_fixture() {
    let root = corpus_root();
    let prepared = run(&root, None, None);
    let explicit = run(&root, None, Some(Effort::Max));
    assert_eq!(
        serde_json::to_string(&prepared.rules).unwrap(),
        serde_json::to_string(&explicit.rules).unwrap()
    );
    assert_eq!(prepared.advisories, explicit.advisories);
    assert!(prepared
        .rules
        .iter()
        .any(|row| { row.rule == RULE && row.target.as_deref() == Some("data-testid=save") }));
    assert!(prepared
        .rules
        .iter()
        .chain(&prepared.advisories)
        .all(|row| !row.message.contains("unanalyzable at effort low")));
}

#[test]
fn low_report_names_the_rewrite_and_keeps_literal_duplicates() {
    let root = effort_root();
    let report = run(&root, Some(".no-mistakes-report.yml"), None);
    let page = "web/app/page.tsx";
    assert_eq!(
        shape_lines(&report.rules, page),
        marked_lines(&read(&root, page), "EXPECT_SHAPE")
    );
    assert_eq!(
        duplicate_targets(&report.rules),
        vec!["data-testid=save", "data-testid=save"]
    );
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);
    assert!(report.rules.iter().any(|row| {
        row.message.contains("unanalyzable at effort low")
            && row.message.contains("literal test id")
            && row.message.contains("prop with a literal default")
    }));
    let source = read(&root, page);
    let silent = source
        .lines()
        .position(|line| line.contains("data-testid={SILENT_ID}"))
        .map(|index| index + 1)
        .unwrap();
    assert!(
        report
            .rules
            .iter()
            .chain(&report.advisories)
            .all(|row| !(row.file == page && row.line == silent)),
        "suppressed identifier must stay silent: {:?}",
        report.rules
    );
}

#[test]
fn low_advise_and_ignore_route_shapes_without_crediting_them() {
    let root = effort_root();
    let advise = run(&root, Some(".no-mistakes-advise.yml"), None);
    let ignore = run(&root, Some(".no-mistakes-ignore.yml"), None);
    let page = "web/app/page.tsx";
    let shapes = marked_lines(&read(&root, page), "EXPECT_SHAPE");
    assert!(shape_lines(&advise.rules, page).is_empty());
    assert_eq!(shape_lines(&advise.advisories, page), shapes);
    assert_eq!(
        duplicate_targets(&advise.rules),
        vec!["data-testid=save", "data-testid=save"]
    );
    assert!(ignore.advisories.is_empty(), "{:?}", ignore.advisories);
    assert!(shape_lines(&ignore.rules, page).is_empty());
    assert_eq!(
        duplicate_targets(&ignore.rules),
        vec!["data-testid=save", "data-testid=save"]
    );
}

#[test]
fn max_fixture_output_matches_and_omits_low_diagnostics() {
    let root = effort_root();
    let default_max = run(&root, Some(".no-mistakes-default-max.yml"), None);
    let explicit_max = run(&root, Some(".no-mistakes-explicit-max.yml"), None);
    assert_eq!(
        serde_json::to_string(&default_max.rules).unwrap(),
        serde_json::to_string(&explicit_max.rules).unwrap()
    );
    assert_eq!(default_max.advisories, explicit_max.advisories);
    assert!(default_max
        .rules
        .iter()
        .chain(&default_max.advisories)
        .all(|row| { !row.message.contains("unanalyzable at effort low") }));
    assert_eq!(
        duplicate_targets(&default_max.rules),
        vec!["data-testid=save", "data-testid=save"]
    );
}
