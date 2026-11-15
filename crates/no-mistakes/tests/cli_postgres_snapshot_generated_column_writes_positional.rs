use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_no_generated_column_writes_positional_advise_snapshot() {
    let rule = "postgres-no-generated-column-writes";
    let config = rules_path(rule, "configs", "positional-advise.yml");
    // Inline SQL merges generated-write and positional-column-order on the
    // executor line. A finite variant keeps that line when the SQL text is
    // declared later. A disable-next-line on the executor suppresses it.
    let rows = [
        ("embedded_advise", "unit-fixture", "positional-embedded"),
        ("variant_advise", "unit-fixture", "positional-variant"),
        (
            "executor_disable",
            "unit-fixture",
            "positional-executor-disable",
        ),
    ];
    let cases =
        rows.map(|(label, kind, name)| (label, rules_path(rule, kind, name), config.clone()));
    let report = assert_saved_fixture_report(
        "postgres_no_generated_column_writes_positional",
        &cases,
        &[],
        &[],
    );
    let advisory = &report["embedded_advise"]["report"]["advisories"];
    assert_eq!(advisory.as_array().unwrap().len(), 1, "{report}");
    assert_eq!(advisory[0]["file"], "write.ts");
    assert_eq!(advisory[0]["line"], 4);
    assert_eq!(
        advisory[0]["sourceLine"],
        "return write(`INSERT INTO votes VALUES ($1, $2)`)"
    );
    let message = advisory[0]["message"].as_str().unwrap();
    assert!(message.contains("generated-write"), "{message}");
    assert!(message.contains("positional-column-order"), "{message}");
    assert!(report["embedded_advise"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
    let variant = &report["variant_advise"]["report"]["advisories"];
    assert_eq!(variant.as_array().unwrap().len(), 1, "{report}");
    assert_eq!(variant[0]["file"], "write.ts");
    assert_eq!(variant[0]["line"], 4);
    assert_eq!(variant[0]["sourceLine"], "return query(");
    let variant_message = variant[0]["message"].as_str().unwrap();
    assert!(
        variant_message.contains("generated-write"),
        "{variant_message}"
    );
    assert!(
        variant_message.contains("positional-column-order"),
        "{variant_message}"
    );
    assert_eq!(report["executor_disable"]["exitCode"], 0);
    assert!(report["executor_disable"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(report["executor_disable"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
}
