use super::*;

#[derive(Clone, Copy)]
pub(super) enum IdentifierUse<'a> {
    Identifier,
    Value,
    Member(&'a [String]),
}

impl<'a> IdentifierUse<'a> {
    pub(super) fn members(self) -> &'a [String] {
        match self {
            Self::Member(members) => members,
            _ => &[],
        }
    }
}

pub(super) fn classify(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    expr: &ValueExpression,
    out: &mut BTreeSet<Provenance>,
    bindings: &Bindings<'_>,
) {
    classify_in(
        graph,
        facts,
        file,
        expr,
        out,
        bindings,
        IdentifierUse::Identifier,
    );
}

pub(super) fn classify_value(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    expr: &ValueExpression,
    out: &mut BTreeSet<Provenance>,
    bindings: &Bindings<'_>,
) {
    classify_in(
        graph,
        facts,
        file,
        expr,
        out,
        bindings,
        IdentifierUse::Value,
    );
}

pub(super) fn classify_in(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    expr: &ValueExpression,
    out: &mut BTreeSet<Provenance>,
    bindings: &Bindings<'_>,
    usage: IdentifierUse<'_>,
) {
    if matches!(usage, IdentifierUse::Value) && bindings.identifier_value_without_usage(file, expr)
    {
        return;
    }
    if super::super::metadata_row_projection(graph, facts, bindings, file, expr, usage.members()) {
        return;
    }
    let env = if usage.members().is_empty() {
        env_key(facts, file, expr).map(str::to_owned).or_else(|| {
            matches!(expr, ValueExpression::Member(_, _))
                .then(|| bindings.env(file, expr, &[]))
                .flatten()
        })
    } else {
        env_from_chain(facts, file, expr, usage.members())
    };
    if let Some(key) = env {
        out.insert(Provenance::ProcessEnv(key.to_string()));
        return;
    }
    match expr {
        ValueExpression::Derived(parts) => {
            for part in parts.iter() {
                classify_in(graph, facts, file, part, out, bindings, usage);
            }
        }
        ValueExpression::Call(start, end) => {
            if let Some(value_call) = value_call_at(facts, file, *start, *end) {
                super::classification_calls::with_context(
                    graph, facts, file, value_call, out, bindings, usage,
                );
            }
        }
        ValueExpression::Awaited(inner) => {
            classify_in(graph, facts, file, inner, out, bindings, usage);
        }
        ValueExpression::Member(inner, key) => {
            if usage.members().is_empty()
                && text_tag::projection_span(bindings, file, expr).is_some()
            {
                return;
            }
            let mut members = vec![key.clone()];
            let mut current = inner.as_ref();
            while let ValueExpression::Member(next, next_key) = current {
                members.push(next_key.clone());
                current = next.as_ref();
            }
            members.reverse();
            members.extend_from_slice(usage.members());
            classify_in(
                graph,
                facts,
                file,
                current,
                out,
                bindings,
                IdentifierUse::Member(&members),
            );
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            classify_in(graph, facts, file, left, out, bindings, usage);
            classify_in(graph, facts, file, right, out, bindings, usage);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            let usage = if usage.members().is_empty() {
                IdentifierUse::Value
            } else {
                usage
            };
            for value in fields.values() {
                classify_in(graph, facts, file, value, out, bindings, usage);
            }
        }
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => {
            out.insert(named_provenance(
                facts,
                file,
                name,
                *scope,
                *offset,
                usage.members(),
            ));
        }
        _ => {}
    }
}
