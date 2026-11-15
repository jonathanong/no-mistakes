use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_no_generated_column_writes_columnless_shapes_snapshot() {
    let rule = "postgres-no-generated-column-writes";
    let root = rules_path(rule, "unit-fixture", "columnless-shapes");
    let rows = [
        ("shapes", "columnless-shapes.yml"),
        ("shapes_ignore", "columnless-shapes-ignore.yml"),
        ("shapes_advise", "columnless-shapes-advise.yml"),
    ];
    let cases =
        rows.map(|(label, config)| (label, root.clone(), rules_path(rule, "configs", config)));
    let report = assert_saved_fixture_report(
        "postgres_no_generated_column_writes_shapes",
        &cases,
        &[],
        &[],
    );

    // Twelve findings, zero advisories: seven unknown-order votes writes, three
    // definite items writes, and two unsupported INSERT ... TABLE statements.
    assert_eq!(source_lines(&report, "shapes"), expected_reported_lines());
    assert_eq!(advisory_count(&report, "shapes"), 0);
    let shapes = &report["shapes"]["report"]["rules"];
    assert_messages(
        shapes,
        &expected_unknown_lines(),
        "unanalyzable",
        "column order is unknown",
    );
    assert_messages(
        shapes,
        &expected_definite_lines(),
        "created_at",
        "items.created_at",
    );
    assert_messages(
        shapes,
        &expected_table_lines(),
        "INSERT INTO",
        "could not be analyzed",
    );

    // Three definite findings, zero advisories. `unanalyzableSql` is the legacy
    // alias of `options.unanalyzable`, so ignore also omits both TABLE diagnostics.
    assert_eq!(
        source_lines(&report, "shapes_ignore"),
        expected_definite_lines()
    );
    assert_eq!(advisory_count(&report, "shapes_ignore"), 0);

    // Three definite findings. Nine advisories: seven positional-column-order
    // votes writes and two INSERT ... TABLE diagnostics.
    assert_eq!(
        source_lines(&report, "shapes_advise"),
        expected_definite_lines()
    );
    let advisories = &report["shapes_advise"]["report"]["advisories"];
    assert_eq!(advisory_count(&report, "shapes_advise"), 9);
    assert_messages(
        advisories,
        &expected_unknown_lines(),
        "positional-column-order",
        "column order is unknown",
    );
    assert_messages(
        advisories,
        &expected_table_lines(),
        "INSERT INTO",
        "could not be analyzed",
    );
}

fn assert_messages(
    findings: &serde_json::Value,
    sources: &[String],
    target_needle: &str,
    message_needle: &str,
) {
    for source in sources {
        let finding = findings
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["sourceLine"].as_str() == Some(source))
            .unwrap_or_else(|| panic!("missing {source}"));
        let target = finding["target"].as_str().unwrap();
        let message = finding["message"].as_str().unwrap();
        assert!(
            target.contains(target_needle) && message.contains(message_needle),
            "{source}: target {target}, message {message}"
        );
    }
}

fn source_lines(report: &serde_json::Value, label: &str) -> Vec<String> {
    let mut lines = report[label]["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["sourceLine"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

fn advisory_count(report: &serde_json::Value, label: &str) -> usize {
    report[label]["report"]["advisories"]
        .as_array()
        .unwrap()
        .len()
}

fn expected_reported_lines() -> Vec<String> {
    let mut lines = expected_unknown_lines();
    lines.extend(expected_definite_lines());
    lines.extend(expected_table_lines());
    lines.sort();
    lines
}

fn expected_unknown_lines() -> Vec<String> {
    [
        "INSERT INTO votes VALUES (1, now()), (2, now());",
        "INSERT INTO votes SELECT 1, now();",
        "INSERT INTO votes SELECT * FROM source_rows;",
        "WITH seed AS (SELECT 1) INSERT INTO votes VALUES (1, now());",
        "WITH written AS (INSERT INTO votes VALUES (1, now()) RETURNING id) SELECT id FROM written;",
        "INSERT INTO votes VALUES (1, now()) ON CONFLICT (id) DO NOTHING;",
        "MERGE INTO votes USING source_rows ON true WHEN NOT MATCHED THEN INSERT VALUES (1, now());",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn expected_definite_lines() -> Vec<String> {
    let mut lines = [
        "INSERT INTO items VALUES (1, 'n', now());",
        "INSERT INTO items SELECT * FROM source_rows;",
        "WITH edited AS (UPDATE items SET created_at = now() RETURNING id) SELECT id FROM edited;",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    lines.sort();
    lines
}

fn expected_table_lines() -> Vec<String> {
    let mut lines = [
        "INSERT INTO votes TABLE source_rows;",
        "INSERT INTO items TABLE source_rows;",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    lines.sort();
    lines
}
