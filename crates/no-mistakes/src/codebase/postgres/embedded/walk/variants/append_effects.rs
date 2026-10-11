use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

type Effect = (String, Option<u32>, Option<usize>);

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn append_alias_effect(
        &self,
        call: &CallExpression<'_>,
    ) -> Vec<Effect> {
        let (receiver, direct) = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                (&member.object, true)
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property) if property.value == "append") => {
                (&member.object, false)
            }
            _ => return Vec::new(),
        };
        let preserved = if direct {
            match unwrap_ts_wrappers(receiver) {
                Expression::Identifier(id) => Some(id.name.as_str()),
                _ => None,
            }
        } else {
            None
        };
        self.builder_result_names(receiver)
            .into_iter()
            .filter_map(|name| {
                let (index, binding) = self
                    .scopes
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, scope)| scope.get(&name).map(|binding| (index, binding)))?;
                // Typed parameters have no recoverable object identity. Merely
                // walking their helper bodies cannot mutate an outer namesake.
                if !binding.initialized {
                    return None;
                }
                let preserved_scope = (preserved == Some(name.as_str())).then_some(index);
                Some((name, binding.builder_identity, preserved_scope))
            })
            .collect()
    }

    pub(in crate::codebase::postgres::embedded::walk) fn apply_append_alias_effect(
        &mut self,
        effects: Vec<Effect>,
    ) {
        for (name, identity, preserved_scope) in effects {
            // Ordinary append invalidates retained variant snapshots only;
            // legacy nonbranching fields keep their existing public behavior.
            self.apply_builder_effect(name, identity, preserved_scope, true, false);
        }
    }
}
