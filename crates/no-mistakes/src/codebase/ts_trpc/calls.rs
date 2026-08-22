use oxc_ast::ast::{CallExpression, Expression};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrpcCallFact {
    pub path: String,
}

pub(crate) fn finish_trpc_calls(calls: &mut Vec<TrpcCallFact>) {
    calls.sort_by(|left, right| left.path.cmp(&right.path));
    calls.dedup_by(|left, right| left.path == right.path);
}

pub(crate) fn procedure_path_from_call(call: &CallExpression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(terminal) = &call.callee else {
        return None;
    };
    if !matches!(
        terminal.property.name.as_str(),
        "query" | "mutate" | "mutation"
    ) {
        return None;
    }
    let mut segments = Vec::new();
    let mut current = &terminal.object;
    loop {
        match current {
            Expression::StaticMemberExpression(member) => {
                segments.push(member.property.name.to_string());
                current = &member.object;
            }
            Expression::Identifier(_) => break,
            _ => return None,
        }
    }
    if segments.len() < 2 {
        return None;
    }
    segments.reverse();
    Some(segments.join("."))
}

#[cfg(test)]
mod tests;
