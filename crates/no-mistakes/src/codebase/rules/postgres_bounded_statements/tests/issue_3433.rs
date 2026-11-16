use super::{check_with_files, config, fixture_root};

struct Case {
    name: String,
    report: bool,
    start: usize,
    end: usize,
}

fn source() -> String {
    std::fs::read_to_string(fixture_root().join("../issue-3433/fixture/statements.sql")).unwrap()
}

fn cases(sql: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut name = String::new();
    let mut report = false;
    let mut start = 1;
    for (index, line) in sql.lines().enumerate() {
        let line_no = index + 1;
        let Some(rest) = line
            .strip_prefix("-- pass ")
            .or_else(|| line.strip_prefix("-- report "))
        else {
            continue;
        };
        if !name.is_empty() {
            cases.push(Case {
                name: std::mem::take(&mut name),
                report,
                start,
                end: line_no - 1,
            });
        }
        report = line.starts_with("-- report ");
        name = rest.to_string();
        start = line_no + 1;
    }
    if !name.is_empty() {
        cases.push(Case {
            name,
            report,
            start,
            end: sql.lines().count(),
        });
    }
    cases
}

#[test]
fn parameter_returning_key_matches_any_array() {
    let sql = source();
    let cases = cases(&sql);
    let root = fixture_root().join("../issue-3433/fixture");
    let findings = check_with_files(
        &root,
        &config(
            "schemaCatalogPath: schema.json\nexecutorNames: []\ninclude: [statements.sql]\nsqlInclude: [statements.sql]\n",
        ),
        &[root.join("statements.sql")],
    )
    .unwrap();
    for case in &cases {
        let reported = findings
            .iter()
            .any(|finding| finding.line >= case.start && finding.line <= case.end);
        assert_eq!(reported, case.report, "{}", case.name);
    }
    assert_eq!(
        findings.len(),
        cases.iter().filter(|case| case.report).count()
    );
    for (name, table) in [
        ("column-recordset", "repro_batches"),
        ("cte-twice", "repro_posts"),
        ("or-join", "repro_posts"),
        ("nonunique-join", "repro_posts"),
    ] {
        let case = cases.iter().find(|case| case.name == name).unwrap();
        let hits: Vec<_> = findings
            .iter()
            .filter(|finding| finding.line >= case.start && finding.line <= case.end)
            .collect();
        assert_eq!(hits.len(), 1, "{name}");
        let expected = format!("table:{table}");
        assert_eq!(hits[0].target.as_deref(), Some(expected.as_str()), "{name}");
        let message = format!("UPDATE can change every row of {table}");
        assert!(
            hits[0].message.contains(&message),
            "{name}: {}",
            hits[0].message
        );
    }
    let from_line = sql
        .lines()
        .position(|line| line.trim() == "FROM repro_batches b")
        .unwrap()
        + 1;
    let column = cases
        .iter()
        .find(|case| case.name == "column-recordset")
        .unwrap();
    let column_hit = findings
        .iter()
        .find(|finding| finding.line >= column.start && finding.line <= column.end)
        .unwrap();
    assert_eq!(column_hit.line, from_line);
}
