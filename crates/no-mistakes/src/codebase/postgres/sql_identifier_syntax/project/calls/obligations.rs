fn append_obligations(selection: &Selection, source: &str, calls: &mut Vec<SqlCall>) {
    calls.extend(selection.reference_obligations.iter().map(|at| SqlCall {
        span: *at,
        line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
        input: Input::Unknown,
    }));
}
