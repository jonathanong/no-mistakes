use super::{check_with_files, config, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn statement_anchor_requires_a_matching_line_directive() {
    use super::super::scan::statement_directive_line;
    let root = fixture_root();
    let files = [
        root.join("sql/suppress.sql"),
        root.join("sql/suppress-file.sql"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let source = sources.read_path(&files[0]).unwrap();
    assert_eq!(statement_directive_line(Some(&source), 7), Some(7));
    assert_eq!(statement_directive_line(Some(&source), 9), Some(9));
    assert_eq!(statement_directive_line(Some(&source), 4), None);
    assert_eq!(statement_directive_line(Some(&source), usize::MAX), None);
    assert_eq!(statement_directive_line(None, 7), None);
    let file_disabled = sources.read_path(&files[1]).unwrap();
    assert_eq!(statement_directive_line(Some(&file_disabled), 2), None);
}

#[test]
fn update_and_delete_facts_start_at_the_statement_keyword() {
    let root = fixture_root();
    let path = root.join("sql/suppress.sql");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let source = sources.read_path(&path).unwrap();
    let bounds = extract_sql_statement_facts(&source).bounds;
    assert_eq!(bounds[5].line, 12);
    assert_eq!(bounds[6].line, 14);
    assert_eq!(bounds[7].line, 17);
    assert_eq!(bounds[8].line, 19);
}

#[test]
fn statement_start_directives_keep_findings_for_the_shared_suppression_layer() {
    let root = fixture_root();
    let files = [root.join("schema.json"), root.join("sql/suppress.sql")];
    let yaml =
        "schemaCatalogPath: schema.json\nsqlInclude: ['sql/suppress.sql']\nexclude: ['src/**']";
    let findings = check_with_files(&root, &config(yaml), &files).unwrap();
    // No finding is discarded early: the shared layer retains suppression audit evidence.
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [2, 3, 4, 7, 9, 12, 14, 17, 19, 22]);
}
