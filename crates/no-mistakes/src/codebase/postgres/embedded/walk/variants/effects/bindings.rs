use super::super::ValueKind;
use super::Effects;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

impl Effects<'_, '_> {
    pub(super) fn known_sql_binding(&self, name: &str) -> bool {
        self.visitor.lookup(name).is_some_and(|binding| {
            binding.sql_builder
                || binding.builder_identity.is_some()
                || binding
                    .variants
                    .as_ref()
                    .is_some_and(|values| values.iter().any(|value| value.value == ValueKind::Sql))
        })
    }
    pub(super) fn appended_builders(
        &self,
        call: &CallExpression<'_>,
    ) -> Vec<(String, Option<u32>)> {
        let receiver = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                &member.object
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property) if property.value == "append") => {
                &member.object
            }
            _ => return Vec::new(),
        };
        self.visitor
            .builder_result_names(receiver)
            .into_iter()
            .filter(|name| self.known_sql_binding(name))
            .map(|name| {
                let identity = self
                    .visitor
                    .lookup(&name)
                    .and_then(|binding| binding.builder_identity);
                (name, identity)
            })
            .collect()
    }
    pub(super) fn collect_binding(&mut self, name: &str) {
        if self.known_sql_binding(name) {
            self.found = true;
            if let Some(prefixes) = &mut self.prefixes {
                prefixes.push((name.to_string(), None, None));
            }
        }
    }
}
