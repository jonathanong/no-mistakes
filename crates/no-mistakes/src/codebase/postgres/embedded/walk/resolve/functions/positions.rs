use super::*;

impl LocalFunctions {
    /// Physical source locations for a parameter helper and each static append.
    /// Returns the complete SQL as well so an enclosing fluent append can
    /// position its own fragment after this helper's text.
    pub(crate) fn call_positions(
        &self,
        call: &CallExpression<'_>,
        name: &str,
        visitor: &ScopeVisitor<'_>,
    ) -> Option<(String, Vec<EmbeddedSqlSourcePosition>, u32)> {
        let builder = self.parameter_builders.get(name)?;
        let argument = builder.checked_argument(call)?;
        let call_line = byte_offset_to_line(visitor.source, call.span.start as usize);
        let (mut sql, mut positions, origin) = match unwrap_ts_wrappers(argument) {
            Expression::Identifier(ident) => {
                let binding = visitor.lookup(ident.name.as_str())?;
                if !binding.sql_builder || binding.kind == EmbeddedSqlKind::Dynamic {
                    return None;
                }
                (binding.sql?, binding.sql_source_positions, binding.line)
            }
            Expression::TaggedTemplateExpression(_) => {
                MAX_RESOLVE_DEPTH.checked_sub(1)?;
                let base = super::super::chain::tagged_sql(
                    argument,
                    &mut |tag| visitor.shadowed_locally(tag) || self.is_tag_shadowed(tag),
                    self.imported_sql_tags(),
                )?;
                let positions = source_positions::for_expression(
                    argument,
                    visitor.source,
                    call.span.start as usize,
                    call_line,
                );
                (base, positions, call_line)
            }
            _ => return None,
        };
        if origin != call_line {
            positions.insert(
                0,
                EmbeddedSqlSourcePosition {
                    sql_line: 1,
                    sql_column: 1,
                    source_line: origin,
                },
            );
        }
        for fragment in &builder.fragments {
            let offset =
                crate::codebase::postgres::embedded::placeholders::count_placeholders(&sql);
            let mut fragment_positions = fragment.positions.clone();
            super::super::append::positions::renumber(
                &mut fragment_positions,
                &fragment.sql,
                offset,
            );
            super::super::append::positions::append(
                &mut positions,
                &sql,
                origin,
                fragment.line,
                &fragment_positions,
            );
            sql.push_str(
                &crate::codebase::postgres::embedded::placeholders::renumber_placeholders(
                    &fragment.sql,
                    offset,
                ),
            );
        }
        Some((sql, positions, origin))
    }
}
