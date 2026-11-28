use super::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use oxc_syntax::operator::AssignmentOperator;

#[path = "instance_properties_ast.rs"]
mod ast;

pub(super) use ast::is_call_derived_construct;
pub use ast::{InstancePropertyEvidence, InstancePropertyValue};

impl<'a> Projector<'a, '_> {
    pub(super) fn collect_instance_properties(
        &mut self,
        call_id: NodeId,
    ) -> Option<InstancePropertyEvidence> {
        let (child_span, parent_id) = ast::non_transparent_parent(self.semantic, call_id);
        match self.semantic.nodes().kind(parent_id) {
            AstKind::ThrowStatement(_) | AstKind::ExpressionStatement(_) => None,
            AstKind::CallExpression(call) => {
                if ast::is_unshadowed_object_assign(self.semantic, &call.callee)
                    && call
                        .arguments
                        .first()
                        .is_some_and(|arg| arg.span() == child_span)
                {
                    let mut ev = InstancePropertyEvidence::default();
                    self.fold_assign_sources(&call.arguments[1..], &mut ev);
                    (!ev.is_empty()).then_some(ev)
                } else {
                    Some(InstancePropertyEvidence::opaque())
                }
            }
            AstKind::VariableDeclarator(decl)
                if decl.init.as_ref().is_some_and(|i| i.span() == child_span) =>
            {
                self.collect_binding_properties(decl, parent_id)
            }
            _ => Some(InstancePropertyEvidence::opaque()),
        }
    }

    fn collect_binding_properties(
        &mut self,
        decl: &VariableDeclarator<'a>,
        decl_id: NodeId,
    ) -> Option<InstancePropertyEvidence> {
        let BindingPattern::BindingIdentifier(binding) = &decl.id else {
            return Some(InstancePropertyEvidence::opaque());
        };
        let symbol = binding.symbol_id.get()?;
        if self.exported.contains(&symbol.index()) {
            return Some(InstancePropertyEvidence::opaque());
        }
        let decl_block = ast::enclosing_block(self.semantic, decl_id);
        let mut ev = InstancePropertyEvidence::default();
        for (ref_id, is_write) in ast::sorted_symbol_refs(self.semantic, symbol) {
            if is_write {
                ev.opaque_mutation = true;
                break;
            }
            self.fold_binding_use(ref_id, decl_block, &mut ev);
        }
        (!ev.is_empty()).then_some(ev)
    }

    fn fold_binding_use(
        &mut self,
        ref_id: NodeId,
        decl_block: Option<NodeId>,
        ev: &mut InstancePropertyEvidence,
    ) {
        let (child_span, use_id) = ast::non_transparent_parent(self.semantic, ref_id);
        match self.semantic.nodes().kind(use_id) {
            AstKind::ThrowStatement(_) => {}
            AstKind::StaticMemberExpression(member) if member.object.span() == child_span => {
                self.fold_member_use(
                    use_id,
                    member.span,
                    Some(member.property.name.as_str()),
                    decl_block,
                    ev,
                );
            }
            AstKind::ComputedMemberExpression(member) if member.object.span() == child_span => {
                let key = ast::static_string_expr(&member.expression);
                self.fold_member_use(use_id, member.span, key, decl_block, ev);
            }
            AstKind::CallExpression(call)
                if ast::is_unshadowed_object_assign(self.semantic, &call.callee)
                    && call
                        .arguments
                        .first()
                        .is_some_and(|a| a.span() == child_span) =>
            {
                if ast::same_unconditional_block(self.semantic, use_id, decl_block) {
                    self.fold_assign_sources(&call.arguments[1..], ev);
                } else {
                    ev.opaque_mutation = true;
                }
            }
            _ => ev.opaque_mutation = true,
        }
    }

    fn fold_member_use(
        &mut self,
        member_id: NodeId,
        member_span: Span,
        key: Option<&str>,
        decl_block: Option<NodeId>,
        ev: &mut InstancePropertyEvidence,
    ) {
        let AstKind::AssignmentExpression(assign) = self.semantic.nodes().parent_kind(member_id)
        else {
            return;
        };
        if assign.left.span() != member_span {
            return;
        }
        if assign.operator != AssignmentOperator::Assign
            || !ast::same_unconditional_block(self.semantic, member_id, decl_block)
        {
            ev.opaque_mutation = true;
            return;
        }
        match key {
            Some(key) => {
                let state = self.classify_delay_value(&assign.right);
                ev.properties.insert(key.to_string(), state);
            }
            None => {
                ev.opaque_mutation = true;
                ev.properties.clear();
            }
        }
    }

    fn fold_assign_sources(
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
                ev.opaque_mutation = true;
                ev.properties.clear();
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

    fn classify_delay_value(&mut self, expr: &Expression<'a>) -> InstancePropertyValue {
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
