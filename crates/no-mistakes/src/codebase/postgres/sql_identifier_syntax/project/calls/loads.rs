fn append_loads(
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    if profile.selects_executors() {
        for load in &module.loads {
            if !profile.import_specifier.is_empty()
                && crate::codebase::postgres::embedded::scoped_bindings::from_configured_module(
                    &load.specifier,
                    &profile.import_specifier,
                )
                && matches!(load.kind.as_str(), "require" | "dynamicImport")
            {
                let at = (load.span.start, load.span.end);
                calls.push(SqlCall {
                    span: at,
                    line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
                    input: Input::Unknown,
                });
            }
        }
    }
}
