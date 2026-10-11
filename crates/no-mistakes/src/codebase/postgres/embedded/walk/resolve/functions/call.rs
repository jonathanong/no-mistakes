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
        self.resolved.get(name).cloned()
    }
}
