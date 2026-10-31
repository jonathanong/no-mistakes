use super::{check_in, findings_of, fixture_dir};

#[test]
fn correlated_builder_predicates_are_judged_in_their_executed_outer_statement() {
    let root = fixture_dir("variants/correlated-append");
    let findings = findings_of(check_in(&root, ".no-mistakes.yml"));
    assert_eq!(
        findings,
        [("src/query.ts".to_string(), 14, "table:accounts".to_string())]
    );
}

#[test]
fn unrecoverable_outer_builder_uses_the_configured_advisory_policy() {
    let root = fixture_dir("variants/correlated-append");
    let output = check_in(&root, "advise.yml");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{report}");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "postgres-bounded-statements"
                && finding["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("unresolved SQL"))),
        "{report}"
    );
}

#[test]
fn correlated_nonunique_column_requires_a_declared_bound() {
    let root = fixture_dir("variants/correlated-append");
    let findings = findings_of(check_in(&root, "undeclared.yml"));
    assert!(
        findings
            .iter()
            .any(|(file, line, target)| file == "src/query.ts"
                && *line == 6
                && target.contains("orders")),
        "{findings:?}"
    );
}

#[test]
fn correlated_append_check_json_snapshot() {
    let root = fixture_dir("variants/correlated-append");
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_bounded_correlated_append",
        &[
            ("recovered", root.clone(), root.join(".no-mistakes.yml")),
            ("undeclared", root.clone(), root.join("undeclared.yml")),
            (
                "unrecoverable_advise",
                root.clone(),
                root.join("advise.yml"),
            ),
        ],
    );
}
