use super::*;
use oxc_span::GetSpan;

impl ImportCollector {
    pub(in super::super) fn note_sql_append_reads(&mut self, call: &CallExpression<'_>) {
        if !self.collect_value_flow || call.optional {
            return;
        }
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };
        if member.property.name != "append" || !self.fresh_default_sql_builder(&member.object) {
            return;
        }
        for argument in &call.arguments {
            let Some(expression) = argument
                .as_expression()
                .map(crate::codebase::ts_source::unwrap_ts_wrappers)
            else {
                continue;
            };
            let Expression::Identifier(identifier) = expression else {
                continue;
            };
            if self.default_sql_builder(expression) {
                self.closed_registry_reads.insert(identifier.span.start);
            }
        }
    }

    fn fresh_default_sql_builder(&self, expression: &Expression<'_>) -> bool {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::TaggedTemplateExpression(tagged) => {
                self.default_sql_tag(&self.value_expression(&tagged.tag))
            }
            Expression::CallExpression(call) if !call.optional => {
                matches!(&call.callee, Expression::StaticMemberExpression(member)
                    if member.property.name == "append" && self.fresh_default_sql_builder(&member.object))
            }
            _ => false,
        }
    }

    pub(in super::super) fn default_sql_builder(&self, expression: &Expression<'_>) -> bool {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::TaggedTemplateExpression(tagged) => {
                self.default_sql_tag(&self.value_expression(&tagged.tag))
            }
            Expression::CallExpression(call) if !call.optional => {
                matches!(&call.callee, Expression::StaticMemberExpression(member)
                    if member.property.name == "append" && self.default_sql_builder(&member.object))
            }
            Expression::Identifier(_) => self.default_sql_value(
                &self.value_expression(expression),
                &mut FxHashSet::default(),
            ),
            _ => false,
        }
    }

    fn default_sql_value(
        &self,
        value: &ValueExpression,
        active: &mut FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                let Some(binding) = self.value_flow.bindings.iter().rev().find(|binding| {
                    binding.name == *name
                        && Some(binding.scope) == *scope
                        && binding.declared_at <= *offset
                }) else {
                    return false;
                };
                binding.immutable
                    && binding.invalidated_at.is_none()
                    && active.insert((binding.scope, binding.name.clone(), binding.declared_at))
                    && self.default_sql_value(&binding.value, active)
            }
            ValueExpression::Call(start, end) => self
                .value_flow
                .calls
                .iter()
                .find(|call| (call.offset, call.end) == (*start, *end))
                .is_some_and(|call| {
                    if self
                        .value_flow
                        .tagged_templates
                        .contains_key(&(*start, *end))
                    {
                        self.default_sql_tag(&call.callee)
                    } else {
                        matches!(&call.callee, ValueExpression::Member(base, method)
                        if method == "append" && self.default_sql_value(base, active))
                    }
                }),
            _ => false,
        }
    }

    fn default_sql_tag(&self, tag: &ValueExpression) -> bool {
        let ValueExpression::Named { name, scope, .. } = tag else {
            return false;
        };
        !scope.is_some_and(|scope| scope != 0)
            && !self
                .value_flow
                .bindings
                .iter()
                .any(|binding| binding.name == *name && binding.scope == scope.unwrap_or(0))
            && self.call_import_bindings.iter().any(|import| {
                !import.is_type_only
                    && import.local == *name
                    && import.specifier == "sql-template-strings"
                    && match import.kind {
                        ImportedBindingKind::Default => true,
                        ImportedBindingKind::Named => {
                            matches!(import.imported.as_str(), "default" | "SQL")
                        }
                        ImportedBindingKind::Namespace => false,
                    }
            })
    }

    pub(in super::super) fn sql_append_member_read(&self, object: &Expression<'_>) -> bool {
        self.closed_registry_call_callee == Some(object.span().start)
            && self.default_sql_builder(object)
    }
}
