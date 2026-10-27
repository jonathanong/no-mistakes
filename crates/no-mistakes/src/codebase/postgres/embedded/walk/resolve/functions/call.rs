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
            return apply_template(template, call, visitor.source);
        }
        super::super::super::imported::recover_call(visitor, call, name)
    }
}

/// Splices a call's string arguments into a template helper.
///
/// A quoted string and a no-substitution template both count. A template with
/// substitutions does not, matching the imported-helper rule.
pub(super) fn apply_template(
    template: &TemplateHelper,
    call: &CallExpression<'_>,
    source: &str,
) -> Option<super::provenance::RecoveredSql> {
    use crate::codebase::postgres::embedded::helpers::split::Marker;
    use crate::codebase::postgres::embedded::helpers::template::Spliced;
    use crate::codebase::ts_source::byte_offset_to_line;
    // An optional call is wrapped in a `ChainExpression`, so it never reaches here.
    let literals = call
        .arguments
        .iter()
        .map(|argument| {
            let (text, start) = literal_argument(argument.as_expression()?)?;
            Some(Spliced {
                text,
                at: Some(Marker {
                    line: byte_offset_to_line(source, start as usize),
                    path: None,
                    origin: start + 1,
                }),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    template.apply(&literals)
}

fn literal_argument<'a>(
    expression: &'a Expression<'a>,
) -> Option<(std::borrow::Cow<'a, str>, u32)> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => {
            Some((crate::ast::js_literal(literal), literal.span.start))
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            template.quasis.first()?.value.cooked.as_ref()?;
            Some((
                crate::ast::template_quasi_text(template.quasis.first()?),
                template.span.start,
            ))
        }
        _ => None,
    }
}
