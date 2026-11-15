use super::{check_with_files, config, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;

struct Case {
    name: String,
    report: bool,
    start: usize,
    end: usize,
    sql: String,
}

fn source() -> String {
    std::fs::read_to_string(fixture_root().join("../issue-3397-reopen/statements.sql")).unwrap()
}

fn cases(sql: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut name = String::new();
    let mut report = false;
    let mut start = 1;
    let mut body = String::new();
    for (index, line) in sql.lines().enumerate() {
        let line_no = index + 1;
        if let Some(rest) = line
            .strip_prefix("-- pass ")
            .or_else(|| line.strip_prefix("-- report "))
        {
            if !name.is_empty() {
                cases.push(Case {
                    name: std::mem::take(&mut name),
                    report,
                    start,
                    end: line_no - 1,
                    sql: std::mem::take(&mut body),
                });
            }
            report = line.starts_with("-- report ");
            name = rest.to_string();
            start = line_no + 1;
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    if !name.is_empty() {
        cases.push(Case {
            name,
            report,
            start,
            end: sql.lines().count(),
            sql: body,
        });
    }
    cases
}

fn candidate_capped(sql: &str) -> bool {
    let extracted = extract_sql_statement_facts(sql);
    assert!(!extracted.parse_failed, "{:?}", extracted.parse_failures);
    extracted.bounds.iter().any(|fact| {
        fact.query.capped
            || fact.query.items.iter().any(|item| {
                matches!(&item.kind, crate::codebase::postgres::SqlBoundItemKind::Query(query) if query.capped)
            })
    })
}

#[test]
fn reopened_shape_keeps_the_limit_and_recall_cases_still_report() {
    let sql = source();
    let cases = cases(&sql);
    let root = fixture_root().join("../issue-3397-reopen");
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
        let limit_trusted = !case.report || case.name == "nonunique-join";
        assert_eq!(candidate_capped(&case.sql), limit_trusted, "{}", case.name);
    }
    assert_eq!(
        findings.len(),
        cases.iter().filter(|case| case.report).count()
    );
}
