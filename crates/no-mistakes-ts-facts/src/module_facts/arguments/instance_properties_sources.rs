use super::super::{ArgumentValue, Projector};
use super::ast::{InstancePropertyEvidence, InstancePropertyValue};
use oxc_ast::ast::{Argument, Expression, ObjectExpression, ObjectPropertyKind, PropertyKind};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;
use serde_json::Value;

impl<'a> Projector<'a, '_> {
    pub(super) fn fold_assign_chain(
        &mut self,
        assign_call_id: NodeId,
        ev: &mut InstancePropertyEvidence,
    ) {
        let AstKind::CallExpression(call) = self.semantic.nodes().kind(assign_call_id) else {
            return;
        };
        self.fold_assign_sources(&call.arguments[1..], ev);
        let (call_span, parent_id) =
            super::ast::non_transparent_parent(self.semantic, assign_call_id);
        if let AstKind::CallExpression(parent) = self.semantic.nodes().kind(parent_id) {
            if super::ast::is_unshadowed_object_assign(self.semantic, &parent.callee)
                && parent
                    .arguments
                    .first()
                    .is_some_and(|arg| arg.span() == call_span)
            {
                self.fold_assign_chain(parent_id, ev);
                return;
            }
        }
        self.absorb_assigned_binding(assign_call_id, ev);
    }

    pub(super) fn fold_assign_sources(
        &mut self,
        sources: &'a [Argument<'a>],
        ev: &mut InstancePropertyEvidence,
    ) {
        for source in sources {
            let Some(entries) = source
                .as_expression()
                .and_then(|expr| self.closed_object_expr(expr, 0))
                .and_then(|object| self.closed_object_delay_entries(object))
            else {
                ev.escape();
                continue;
            };
            for (key, value) in entries {
                ev.properties.insert(key, value);
            }
        }
    }

    fn closed_object_delay_entries(
        &mut self,
        object: &'a ObjectExpression<'a>,
    ) -> Option<Vec<(String, InstancePropertyValue)>> {
        let mut entries = Vec::with_capacity(object.properties.len());
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            let key =
                (property.kind == PropertyKind::Init && !property.method && !property.computed)
                    .then(|| crate::ast::property_key_name(&property.key))
                    .flatten()
                    .filter(|k| k != "__proto__" && k.len() <= 128)?;
            entries.push((key.into_owned(), self.classify_delay_value(&property.value)));
        }
        Some(entries)
    }

    pub(super) fn classify_delay_value(&mut self, expr: &Expression<'a>) -> InstancePropertyValue {
        if self.is_absent_or_non_finite(expr, 0) {
            return InstancePropertyValue::AbsentOrNonFinite;
        }
        match self.value(expr, 0) {
            ArgumentValue::Scalar(Value::Number(num))
                if num.as_f64().is_some_and(|n| n.is_finite() && n > 0.0) =>
            {
                InstancePropertyValue::Positive
            }
            ArgumentValue::Unknown => InstancePropertyValue::Positive,
            _ => InstancePropertyValue::Invalid,
        }
    }
}
