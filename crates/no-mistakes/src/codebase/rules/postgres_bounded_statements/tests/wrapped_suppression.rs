use super::{check_with_files, config, fixture_root};

#[test]
fn wrapped_statement_directives_anchor_and_audit_the_executed_write() {
    let root = fixture_root();
    let path = root.join("sql/wrapped-statement-suppression.sql");
    let files = [root.join("schema.json"), path.clone()];
    let yaml = "schemaCatalogPath: schema.json\nsqlInclude: ['sql/wrapped-statement-suppression.sql']\nexclude: ['src/**']";
    let mut findings = check_with_files(&root, &config(yaml), &files).unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [2, 6, 9, 13, 17, 19]);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let source = sources.read_path(&path).unwrap();
    crate::codebase::rules::suppression::suppress_rule_findings_with_source(&mut findings, &source);
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [17, 19]);
}

#[test]
fn embedded_wrapped_directives_keep_the_original_operand_line() {
    let root = fixture_root();
    let path = root.join("src/wrapped-statement-suppression.mts");
    let files = [root.join("schema.json"), path.clone()];
    let yaml = "schemaCatalogPath: schema.json\nexecutorCalls: [query]\nsqlInclude: []";
    let mut findings = check_with_files(&root, &config(yaml), &files).unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [5, 6, 9, 11]);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let source = sources.read_path(&path).unwrap();
    crate::codebase::rules::suppression::suppress_rule_findings_with_source(&mut findings, &source);
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [9, 11]);
}

#[test]
fn bound_origins_preserve_write_positions_and_optional_mapping() {
    let root = fixture_root();
    let path = root.join("sql/wrapped-statement-suppression.sql");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let source = sources.read_path(&path).unwrap();
    let mut bounds = crate::codebase::postgres::extract_sql_statement_facts(&source).bounds;
    assert_eq!(bounds[0].line, 3);
    assert_eq!(bounds[0].statement_start, Some((2, 1)));
    bounds[0].map_lines(&|line, column| line + column + 20);
    assert_eq!(bounds[0].line, 24);
    assert_eq!(bounds[0].statement_start, Some((23, 1)));
    bounds[1].statement_start = None;
    bounds[1].map_lines(&|line, _| line + 20);
    assert_eq!(bounds[1].line, 27);
    assert_eq!(bounds[1].statement_start, None);
}
