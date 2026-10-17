use super::*;

impl LocalFunctions {
    pub(crate) fn get_call_recovered(
        &self,
        call: &CallExpression<'_>,
        name: &str,
        visitor: &ScopeVisitor<'_>,
    ) -> Option<super::provenance::RecoveredSql> {
        if self.parameter_builders.contains_key(name) {
            let (sql, positions, line) = self.call_positions(call, name, visitor)?;
            return Some(super::provenance::RecoveredSql {
                origins: vec![call.span.start; sql.len()],
                sql,
                line,
                positions,
            });
        }
        if let Some(recovered) = self.resolved.get(name) {
            return Some(recovered.clone());
        }
        if let Some(template) = self.templates.get(name) {
            return apply_template(template, call);
        }
        super::super::super::imported::recover_call(visitor, call, name)
    }
}

/// Splices a call's string-literal arguments into a template helper.
pub(super) fn apply_template(
    template: &TemplateHelper,
    call: &CallExpression<'_>,
) -> Option<super::provenance::RecoveredSql> {
    if call.optional {
        return None;
    }
    let literals = call
        .arguments
        .iter()
        .map(
            |argument| match unwrap_ts_wrappers(argument.as_expression()?) {
                Expression::StringLiteral(literal) => Some(literal.value.as_str()),
                _ => None,
            },
        )
        .collect::<Option<Vec<_>>>()?;
    template.apply(&literals)
}
