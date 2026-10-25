fn append_runtime(
    selection: &Selection,
    syntax: &Syntax,
    direct: &FxHashSet<Span>,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    calls.extend(selection.runtime.iter().filter_map(|at| {
        if selection.transports.contains(at)
            || syntax.metadata.belongs_to(at, direct, &syntax.template_binds)
        {
            return None;
        }
        syntax.metadata.runtime_callsite(at)?;
        Some(SqlCall {
            span: *at,
            line: syntax.metadata.line(at, source),
            input: Input::Unknown,
        })
    }));
}
