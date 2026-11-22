use super::super::load_windows::direct_module_load;
use super::super::{fact_reads, text};
use crate::codebase::ts_source::facts::TsFileFacts;

#[test]
fn text_readers_keep_static_module_and_export_names() {
    assert_eq!(text::quoted("\"columns\""), Some("columns"));
    assert_eq!(text::quoted("'columns'"), Some("columns"));
    assert_eq!(text::quoted("columns"), None);
    assert_eq!(text::quoted("\""), None);
    assert_eq!(text::whole_quoted("  \"columns\"  "), Some("columns"));
    assert_eq!(text::whole_quoted("\"columns\").x"), Some("columns"));
    assert_eq!(text::whole_quoted("\"columns\","), Some("columns"));
    assert_eq!(text::whole_quoted("\"columns\".x"), Some("columns"));
    assert_eq!(text::whole_quoted("\"columns\" + x"), None);
    assert_eq!(
        text::declarator_text("widgetColumns, orderColumns"),
        "widgetColumns"
    );
    assert_eq!(
        text::declarator_text("foo(widgetColumns, orderColumns)"),
        "foo(widgetColumns, orderColumns)"
    );
    assert_eq!(
        text::declarator_text("widgetColumns; orderColumns"),
        "widgetColumns"
    );
    assert_eq!(
        text::declarator_text("widgetColumns\norderColumns"),
        "widgetColumns"
    );
    assert_eq!(
        text::declarator_text("{ widgetColumns, orderColumns }"),
        "{ widgetColumns, orderColumns }"
    );

    let alias = "export { widgetColumns as orderColumns }\nexport {";
    assert_eq!(
        text::export_alias_local(alias, "orderColumns"),
        Some("widgetColumns")
    );
    assert_eq!(
        text::export_alias_local("export { widgetColumns }", "widgetColumns"),
        Some("widgetColumns")
    );
    assert_eq!(text::export_alias_local(alias, "missing"), None);
    assert_eq!(text::export_alias_local("export {", "widgetColumns"), None);

    let imports = "const widgetColumns = 1;\nimport { other } from \"./other\";\nimport { widgetColumns } from \"./columns\";";
    assert_eq!(
        text::import_specifier(imports, "widgetColumns"),
        Some("./columns")
    );
    assert_eq!(
        text::import_specifier("import { widgetColumns }", "widgetColumns"),
        None
    );
    assert_eq!(
        text::import_specifier("const widgetColumns = 1;", "widgetColumns"),
        None
    );

    let equals = "import widgetColumns = nodot;\nimport widgetColumns = columns.a-b;\nimport widgetColumns = columns.widgetColumns;";
    assert_eq!(
        text::import_equals_member(equals, "widgetColumns"),
        Some(("columns", "widgetColumns"))
    );
    assert_eq!(
        text::import_alias("import columns = widgets;", "columns"),
        Some("widgets")
    );
    assert_eq!(
        text::import_alias("import columns = widgets.item;", "columns"),
        None
    );
    assert_eq!(
        text::import_alias("import columns = 1bad;", "columns"),
        None
    );

    assert_eq!(
        text::import_call_specifier("import(\"./columns\").then(ns)"),
        Some("./columns")
    );
    assert_eq!(
        text::import_call_specifier("require(\"./columns\")"),
        Some("./columns")
    );
    assert_eq!(text::import_call_specifier("import(name)"), None);
}

#[test]
fn literal_require_needs_a_recorded_specifier() {
    let [literal, _, _] = module_load_windows();
    let mut facts = TsFileFacts::default();
    facts
        .namespaces
        .literal_import_spans
        .push((0, "./resolved-columns".to_string()));
    assert_eq!(
        fact_reads::depth_zero_module_specifier(&facts, literal, 0).as_deref(),
        Some("./resolved-columns")
    );

    assert!(!fact_reads::unrecorded_static_specifier(&facts, literal, 0));

    facts.namespaces.literal_import_spans.clear();
    assert_eq!(
        fact_reads::depth_zero_module_specifier(&facts, literal, 0),
        None
    );
    assert!(fact_reads::unrecorded_static_specifier(&facts, literal, 0));
}

#[test]
fn direct_module_load_requires_a_complete_call_to_the_exact_callee() {
    let [literal, helper, escaped] = module_load_windows();
    assert!(direct_module_load(literal, false));
    assert!(!direct_module_load(helper, false));
    assert!(direct_module_load(escaped, false));

    let (open_comment, _) = escaped.split_once("*/").expect("block comment");
    assert!(!direct_module_load(open_comment, false));

    let open_call = escaped.strip_suffix(')').expect("closing parenthesis");
    assert!(!direct_module_load(open_call, false));
}

fn module_load_windows() -> [&'static str; 3] {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/fact-readers/module-load-windows.cjs"
    ));
    let mut lines = source.lines().map(|line| line.trim_end_matches(';'));
    std::array::from_fn(|_| lines.next().expect("module load fixture line"))
}

#[test]
fn module_gap_windows_ignore_lexical_decoys_and_accept_static_templates() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/fact-readers/loader-windows.mts"
    ));
    let ts = crate::codebase::ts_source::facts::TsFileFacts::default();
    let expected = [
        false, false, false, true, true, false, true, false, false, true, true, false, true, true,
    ];
    let lines: Vec<_> = source.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), expected.len());
    for (line, expected) in lines.into_iter().zip(expected) {
        assert_eq!(
            fact_reads::unrecorded_static_specifier(&ts, line, 0),
            expected,
            "{line}"
        );
    }
}

#[test]
fn module_gap_windows_cover_escaped_and_incomplete_lexical_input() {
    let cases: Vec<(String, bool)> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/fact-readers/loader-edge-windows.json"
    ))).expect("saved lexical cases");
    let ts = TsFileFacts::default();
    for (source, expected) in cases {
        assert_eq!(
            fact_reads::unrecorded_static_specifier(&ts, &source, 0),
            expected,
            "{source}"
        );
    }
}

#[test]
fn recorded_load_lookup_advances_past_quoted_spans() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-fragment-escape/fixtures/fact-readers/recorded-after-string.ts"
    ));
    let mut ts = TsFileFacts::default();
    let offset = u32::try_from(source.find("load(").expect("saved call")).unwrap();
    ts.namespaces.literal_import_spans = vec![(1, "./decoy".into()), (offset, "./columns".into())];
    assert_eq!(
        fact_reads::depth_zero_module_specifier(&ts, source, 0).as_deref(),
        Some("./columns")
    );
    assert!(!fact_reads::unrecorded_static_specifier(&ts, source, 0));
}
