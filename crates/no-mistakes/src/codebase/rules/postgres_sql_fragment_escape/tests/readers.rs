use super::super::text;

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
        text::export_offset("export widgetColumns", "widgetColumns"),
        Some(0)
    );
    assert_eq!(
        text::export_offset("reexport widgetColumns", "widgetColumns"),
        None
    );
    assert_eq!(
        text::export_offset("export { widgetColumns as orderColumns }", "widgetColumns"),
        Some(0)
    );
    assert_eq!(
        text::export_offset("export {\n  widgetColumns\n};", "widgetColumns"),
        Some(0)
    );
    assert_eq!(
        text::export_offset("export import { widgetColumns }", "widgetColumns"),
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
