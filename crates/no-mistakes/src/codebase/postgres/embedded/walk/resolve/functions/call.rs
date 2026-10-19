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

/// Splices a call's string-literal arguments into a template helper.
///
/// Only a `StringLiteral` counts. A no-substitution template literal is not
/// spliced, matching the imported-helper rule.
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
            let Expression::StringLiteral(literal) = unwrap_ts_wrappers(argument.as_expression()?)
            else {
                return None;
            };
            Some(Spliced {
                text: literal.value.as_str(),
                at: Some(Marker {
                    line: byte_offset_to_line(source, literal.span.start as usize),
                    path: None,
                    origin: literal.span.start + 1,
                }),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    template.apply(&literals)
}
