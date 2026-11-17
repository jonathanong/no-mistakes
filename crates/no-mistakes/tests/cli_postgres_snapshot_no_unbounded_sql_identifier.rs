use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::{Path, PathBuf};

#[test]
fn postgres_guard_provenance_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-guards",
    );
    assert_saved_fixture(
        "postgres_guard_provenance",
        &[("guards", root.clone(), root.join(".no-mistakes.yml"))],
    );
    assert_saved_fixture(
        "postgres_identifier_guards_clean",
        &[("max", root.clone(), root.join(".no-mistakes-clean.yml"))],
    );
}

#[test]
fn low_values_executors_check_json_snapshot() {
    let root = rules_path("postgres-no-offset", "fixture", "low-values-executors");
    assert_saved_fixture(
        "postgres_low_values_executors",
        &[
            ("low", root.clone(), root.join(".no-mistakes.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

fn push(
    cases: &mut Vec<(String, PathBuf, PathBuf)>,
    rule: &str,
    config: &Path,
    scenarios: &[&str],
) {
    for scenario in scenarios {
        cases.push((
            scenario.replace('-', "_"),
            rules_path(rule, "fixture", scenario),
            config.to_path_buf(),
        ));
    }
}

#[test]
fn postgres_no_unbounded_sql_identifier_check_json_snapshot() {
    let rule = "postgres-no-unbounded-sql-identifier";
    let configs = [
        rules_path(rule, "configs", "registry.yml"),
        rules_path(rule, "configs", "factory.yml"),
        rules_path(rule, "configs", "finite-domain-assertions.yml"),
    ];
    let mut owned = Vec::new();
    extend_identifier_cases_1(&mut owned, rule, &configs);
    extend_identifier_cases_2(&mut owned, rule, &configs);
    let cases: Vec<(&str, PathBuf, PathBuf)> = owned
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect();
    assert_saved_fixture("postgres_no_unbounded_sql_identifier", &cases);
}

#[test]
fn postgres_low_typed_transports_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-typed-transports",
    );
    assert_saved_fixture(
        "postgres_low_typed_transports",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

#[test]
fn postgres_low_recovery_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-recovery",
    );
    assert_saved_fixture(
        "postgres_low_recovery",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

#[test]
fn postgres_low_await_using_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-await-using",
    );
    assert_saved_fixture(
        "postgres_low_await_using",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

#[test]
fn postgres_identifier_fragments_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-fragments",
    );
    for shape in ["text", "joins", "union", "negative", "raw"] {
        assert_saved_fixture(
            &format!("postgres_identifier_fragments_{shape}"),
            &[(
                "max",
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
}

include!("cli_postgres_identifier_cases_part1.rs");
include!("cli_postgres_identifier_cases_part2.rs");

#[test]
fn postgres_identifier_aliased_bind_pairing_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3525",
    );
    let report = super::check_json_snapshot::assert_saved_fixture_report(
        "postgres_identifier_aliased_bind_pairing",
        &[("aliases", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["aliases"]["report"]["rules"].as_array().unwrap();
    let mut lines = findings
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(lines, [7, 20, 27, 51, 59, 77]);
    for finding in findings {
        assert_eq!(
            finding["target"],
            if finding["line"] == 77 { "table" } else { "q" }
        );
    }
}
