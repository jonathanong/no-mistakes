fn append_runtime(
    selection: &Selection,
    syntax: &Syntax,
    direct: &FxHashSet<Span>,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    calls.extend(selection.runtime.iter()
        .filter(|at| !syntax.metadata.belongs_to(at, direct, &syntax.template_binds))
        .map(|at| SqlCall {
            span: *at,
            line: syntax.metadata.line(at, source),
            input: Input::Unknown,
        }));
}
