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
            line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
            input: Input::Unknown,
        }));
}
