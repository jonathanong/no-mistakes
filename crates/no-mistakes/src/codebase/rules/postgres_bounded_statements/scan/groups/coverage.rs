use crate::codebase::postgres::AllowList;
use crate::codebase::rules::RuleFinding;

#[test]
fn region_a_finding_without_a_target_stays_out_of_the_group() {
    let mut output = crate::codebase::rules::PostgresFindings::default();
    output.push(RuleFinding {
        rule: "postgres-bounded-statements".into(),
        file: "missing.sql".into(),
        line: 1,
        message: "missing.sql:1: hi".into(),
        import: None,
        target: None,
        causes: Vec::new(),
    });
    let allow = AllowList::compile("postgres-bounded-statements", Vec::new()).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let combined = super::merge(
        &mut output,
        ("exec.ts", None),
        &[],
        std::path::Path::new("."),
        &sources,
        &allow,
    );
    assert!(combined.is_none());
    assert_eq!(output.0.len(), 1);
    assert!(output.0[0].finding.target.is_none());
}
