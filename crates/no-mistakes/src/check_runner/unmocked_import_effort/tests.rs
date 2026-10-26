use super::{run_all_with_options, AnalysisOptions};
use no_mistakes::config::v2::load_v2_config;
use no_mistakes::config::v2::Effort;
use std::path::{Path, PathBuf};

const RULE: &str = "test-no-unmocked-dynamic-imports";

fn effort_root() -> PathBuf {
    // `check()` relativizes against the root it is given. A `..` root does not
    // share a prefix with the normalized files the runner analyzes.
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-no-unmocked-dynamic-imports/fixture/effort-low"),
    )
}

fn corpus_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    )
}

fn run(
    root: &Path,
    config: Option<&str>,
    effort: Option<Effort>,
) -> crate::check_runner::CheckResults {
    let config_path = config.map(|name| root.join(name));
    crate::test_support::policy_advisories(
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
        .unwrap_or_else(|error| panic!("{}: {error}", root.display())),
        "test-no-unmocked-dynamic-imports",
    )
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

fn unmocked_lines(rows: &[no_mistakes::codebase::rules::RuleFinding], file: &str) -> Vec<usize> {
    lines_where(rows, file, "must be mocked")
}

#[test]
fn prepared_max_matches_unprepared_output_on_the_existing_fixture() {
    let root = corpus_root();
    let config = load_v2_config(&root, None).unwrap();
    let unprepared =
        no_mistakes::codebase::rules::test_no_unmocked_dynamic_imports::check(&root, &config, None)
            .unwrap();
    let prepared = run(&root, None, None);
    let explicit = run(&root, None, Some(Effort::Max));
    let prepared_json = serde_json::to_string(&prepared.rules).unwrap();
    let explicit_json = serde_json::to_string(&explicit.rules).unwrap();
    let unprepared_json = serde_json::to_string(&unprepared).unwrap();
    assert_eq!(prepared_json, explicit_json);
    assert_eq!(prepared_json, unprepared_json);
    assert_eq!(prepared.advisories, explicit.advisories);
    assert!(prepared.rules.iter().all(|row| row.rule == RULE));
    assert!(prepared
        .rules
        .iter()
        .chain(&prepared.advisories)
        .all(|row| !row.message.contains("unanalyzable at effort low")));
}

#[test]
fn low_report_names_the_rewrite_and_keeps_uncredited_imports() {
    let root = effort_root();
    let report = run(&root, Some(".no-mistakes-report.yml"), None);
    assert_eq!(
        shape_lines(&report.rules, "src/setup.mts"),
        marked_lines(&read(&root, "src/setup.mts"), "EXPECT_SHAPE")
    );
    assert_eq!(
        shape_lines(&report.rules, "src/require-bridge.cjs"),
        marked_lines(&read(&root, "src/require-bridge.cjs"), "EXPECT_SHAPE")
    );
    assert_eq!(
        unmocked_lines(&report.rules, "src/conditional.test.mts"),
        marked_lines(&read(&root, "src/conditional.test.mts"), "EXPECT_UNMOCKED")
    );
    assert_eq!(
        unmocked_lines(&report.rules, "src/do-mock.test.mts"),
        marked_lines(&read(&root, "src/do-mock.test.mts"), "EXPECT_UNMOCKED")
    );
    assert!(unmocked_lines(&report.rules, "src/accepted.test.mts").is_empty());
    assert!(report.advisories.is_empty(), "{:?}", report.advisories);
    assert!(report.rules.iter().any(|row| {
        row.message.contains("unanalyzable at effort low")
            && row.message.contains("top-level static import")
            && row.message.contains("vi.mock")
            && row.message.contains("literal require()")
    }));
    let setup = read(&root, "src/setup.mts");
    for line in marked_lines(&setup, "EXPECT_SILENT")
        .into_iter()
        .chain(marked_lines(&setup, "suppressed-dynamic"))
    {
        assert!(
            report
                .rules
                .iter()
                .chain(&report.advisories)
                .all(|row| { !(row.file == "src/setup.mts" && row.line == line) }),
            "line {line} must stay silent: {:?}",
            report.rules
        );
    }
}

#[test]
fn low_advise_and_ignore_route_shapes_without_crediting_them() {
    let root = effort_root();
    let advise = run(&root, Some(".no-mistakes-advise.yml"), None);
    let ignore = run(&root, Some(".no-mistakes-ignore.yml"), None);
    let setup = marked_lines(&read(&root, "src/setup.mts"), "EXPECT_SHAPE");
    let bridge = marked_lines(&read(&root, "src/require-bridge.cjs"), "EXPECT_SHAPE");
    assert!(shape_lines(&advise.rules, "src/setup.mts").is_empty());
    assert_eq!(shape_lines(&advise.advisories, "src/setup.mts"), setup);
    assert_eq!(
        shape_lines(&advise.advisories, "src/require-bridge.cjs"),
        bridge
    );
    assert_eq!(
        unmocked_lines(&advise.rules, "src/conditional.test.mts"),
        marked_lines(&read(&root, "src/conditional.test.mts"), "EXPECT_UNMOCKED")
    );
    assert_eq!(
        unmocked_lines(&advise.rules, "src/do-mock.test.mts"),
        marked_lines(&read(&root, "src/do-mock.test.mts"), "EXPECT_UNMOCKED")
    );
    assert!(ignore.advisories.is_empty(), "{:?}", ignore.advisories);
    assert!(shape_lines(&ignore.rules, "src/setup.mts").is_empty());
    assert!(shape_lines(&ignore.rules, "src/require-bridge.cjs").is_empty());
    assert_eq!(
        unmocked_lines(&ignore.rules, "src/do-mock.test.mts"),
        marked_lines(&read(&root, "src/do-mock.test.mts"), "EXPECT_UNMOCKED")
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
    assert!(unmocked_lines(&default_max.rules, "src/conditional.test.mts").is_empty());
    assert!(unmocked_lines(&default_max.rules, "src/do-mock.test.mts").is_empty());
    assert!(unmocked_lines(&default_max.rules, "src/accepted.test.mts").is_empty());
}
