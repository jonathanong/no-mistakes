fn append_runtime(
    selection: &Selection,
    syntax: &Syntax,
    direct: &FxHashSet<Span>,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    let ordinary = calls.iter().map(|call| call.span).collect::<FxHashSet<_>>();
    let mut callees = FxHashMap::<Span, Span>::default();
    let mut operands = Vec::new();
    for at in &selection.runtime {
        if selection.transports.contains(at)
            || syntax
                .metadata
                .belongs_to(at, direct, &syntax.template_binds)
        {
            continue;
        }
        if syntax.metadata.runtime_callsite(at).is_none() {
            continue;
        }
        if let Some(call) = syntax.metadata.runtime_callee(at) {
            if ordinary.contains(&call) {
                continue;
            }
            callees
                .entry(call)
                .and_modify(|candidate| {
                    if (at.1 - at.0, *at) < (candidate.1 - candidate.0, *candidate) {
                        *candidate = *at;
                    }
                })
                .or_insert(*at);
        } else {
            operands.push(*at);
        }
    }
    operands.extend(callees.into_values());
    calls.extend(operands.into_iter().map(|at| SqlCall {
        span: at,
        line: syntax.metadata.line(&at, source),
        input: Input::Unknown,
    }));
}
