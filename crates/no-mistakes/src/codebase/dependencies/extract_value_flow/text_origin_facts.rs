use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextOrigin {
    pub(crate) literal: bool,
    pub(crate) parts: Vec<Option<TextOrigin>>,
    pub(crate) fields: FxHashMap<String, TextOrigin>,
}

#[cfg(test)]
#[path = "text_origin_facts_tests.rs"]
mod tests;

type BindingOrigins = FxHashMap<usize, FxHashMap<String, FxHashMap<u32, TextOrigin>>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextOrigins {
    arguments: FxHashMap<(u32, u32), Vec<Option<TextOrigin>>>,
    bindings: BindingOrigins,
}

impl TextOrigins {
    pub(crate) fn argument(&self, span: (u32, u32), index: usize) -> Option<&TextOrigin> {
        self.arguments.get(&span)?.get(index)?.as_ref()
    }

    pub(crate) fn binding(&self, scope: usize, name: &str, at: u32) -> Option<&TextOrigin> {
        self.bindings.get(&scope)?.get(name)?.get(&at)
    }
}

impl ImportCollector {
    pub(in super::super) fn note_text_call_origins(
        &mut self,
        call: &CallExpression<'_>,
        value: &ValueCall,
    ) {
        if value.bound
            || value.arguments.len() != call.arguments.len()
            || value.callee != self.value_expression(&call.callee)
        {
            return;
        }
        let arguments: Vec<_> = call
            .arguments
            .iter()
            .zip(value.arguments.iter())
            .map(|(raw, value)| raw.as_expression().and_then(|raw| origin(raw, value)))
            .collect();
        if arguments.iter().any(Option::is_some) {
            self.text_origins
                .arguments
                .insert((call.span.start, call.span.end), arguments);
        }
    }

    pub(in super::super) fn note_text_binding_origin(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        at: u32,
        value: &ValueExpression,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        let Some(origin) = origin(init, value) else {
            return;
        };
        let scope = self
            .callee_binding_scope(identifier.name.as_str())
            .unwrap_or(0);
        self.text_origins
            .bindings
            .entry(scope)
            .or_default()
            .entry(identifier.name.to_string())
            .or_default()
            .insert(at, origin);
    }
}

fn origin(expr: &Expression<'_>, value: &ValueExpression) -> Option<TextOrigin> {
    let expr = crate::codebase::ts_source::unwrap_ts_wrappers(expr);
    if matches!(value, ValueExpression::Unknown) {
        return scalar_literal(expr).then(|| TextOrigin {
            literal: true,
            ..Default::default()
        });
    }
    match (expr, value) {
        (Expression::TemplateLiteral(template), ValueExpression::Derived(values)) => {
            parts(&template.expressions.iter().collect::<Vec<_>>(), values)
        }
        (Expression::BinaryExpression(binary), ValueExpression::Derived(values)) => {
            parts(&[&binary.left, &binary.right], values)
        }
        (Expression::ConditionalExpression(branch), ValueExpression::Derived(values)) => {
            parts(&[&branch.consequent, &branch.alternate], values)
        }
        (Expression::LogicalExpression(logical), ValueExpression::Nullish(left, right)) => {
            let values = vec![origin(&logical.left, left), origin(&logical.right, right)];
            retained_parts(values)
        }
        (Expression::AwaitExpression(awaited), ValueExpression::Awaited(value)) => {
            origin(&awaited.argument, value)
        }
        (Expression::StaticMemberExpression(member), ValueExpression::Member(value, _)) => {
            origin(&member.object, value)
        }
        (Expression::ComputedMemberExpression(member), ValueExpression::Member(value, _)) => {
            origin(&member.object, value)
        }
        (Expression::ObjectExpression(object), ValueExpression::Object(values, _)) => {
            let mut fields = FxHashMap::default();
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    fields.clear();
                    continue;
                };
                if property.method || property.kind != oxc_ast::ast::PropertyKind::Init {
                    return None;
                }
                if property.computed {
                    fields.clear();
                    continue;
                }
                let Some(key) = write_targets::property_key(&property.key) else {
                    fields.clear();
                    continue;
                };
                if let Some(origin) = values
                    .get(&key)
                    .and_then(|value| origin(&property.value, value))
                {
                    fields.insert(key, origin);
                } else {
                    fields.remove(&key);
                }
            }
            (!fields.is_empty()).then_some(TextOrigin {
                fields,
                ..Default::default()
            })
        }
        (Expression::ArrayExpression(array), ValueExpression::Array(values)) => {
            let mut fields = FxHashMap::default();
            for (index, element) in array.elements.iter().enumerate() {
                let key = index.to_string();
                if let Some(origin) = element
                    .as_expression()
                    .and_then(|expr| values.get(&key).and_then(|value| origin(expr, value)))
                {
                    fields.insert(key, origin);
                }
            }
            (!fields.is_empty()).then_some(TextOrigin {
                fields,
                ..Default::default()
            })
        }
        _ => None,
    }
}

fn parts(exprs: &[&Expression<'_>], values: &[ValueExpression]) -> Option<TextOrigin> {
    if exprs.len() != values.len() {
        return None;
    }
    retained_parts(
        exprs
            .iter()
            .zip(values)
            .map(|(expr, value)| origin(expr, value))
            .collect(),
    )
}

fn retained_parts(parts: Vec<Option<TextOrigin>>) -> Option<TextOrigin> {
    parts.iter().any(Option::is_some).then_some(TextOrigin {
        parts,
        ..Default::default()
    })
}

fn scalar_literal(expr: &Expression<'_>) -> bool {
    if super::literal_argument(expr) {
        return true;
    }
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::ConditionalExpression(branch) => {
            scalar_literal(&branch.consequent) && scalar_literal(&branch.alternate)
        }
        _ => false,
    }
}
