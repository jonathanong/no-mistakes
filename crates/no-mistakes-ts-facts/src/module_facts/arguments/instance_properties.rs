use super::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use oxc_syntax::operator::AssignmentOperator;

#[path = "instance_properties_ast.rs"]
mod ast;

#[path = "instance_properties_sources.rs"]
mod sources;

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
            AstKind::CallExpression(_) => {
                let receives_assign = matches!(
                    self.semantic.nodes().kind(parent_id),
                    AstKind::CallExpression(call)
                        if ast::is_unshadowed_object_assign(self.semantic, &call.callee)
                            && call
                                .arguments
                                .first()
                                .is_some_and(|arg| arg.span() == child_span)
                );
                if receives_assign {
                    let mut ev = InstancePropertyEvidence::default();
                    self.fold_assign_chain(parent_id, &mut ev);
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
        let mut ev = InstancePropertyEvidence::default();
        self.fold_bound_declarator(decl, decl_id, &mut ev);
        (!ev.is_empty()).then_some(ev)
    }

    fn absorb_assigned_binding(
        &mut self,
        assign_call_id: NodeId,
        ev: &mut InstancePropertyEvidence,
    ) {
        let (call_span, parent_id) = ast::non_transparent_parent(self.semantic, assign_call_id);
        let AstKind::VariableDeclarator(decl) = self.semantic.nodes().kind(parent_id) else {
            return;
        };
        if decl
            .init
            .as_ref()
            .is_some_and(|init| init.span() == call_span)
        {
            self.fold_bound_declarator(decl, parent_id, ev);
        }
    }

    fn fold_bound_declarator(
        &mut self,
        decl: &VariableDeclarator<'a>,
        decl_id: NodeId,
        ev: &mut InstancePropertyEvidence,
    ) {
        let BindingPattern::BindingIdentifier(binding) = &decl.id else {
            ev.escape();
            return;
        };
        let Some(symbol) = binding.symbol_id.get() else {
            ev.escape();
            return;
        };
        if self.exported.contains(&symbol.index()) {
            ev.escape();
            return;
        }
        let decl_block = ast::enclosing_block(self.semantic, decl_id);
        for (ref_id, is_write) in ast::sorted_symbol_refs(self.semantic, symbol) {
            if is_write {
                ev.opaque_mutation = true;
                break;
            }
            self.fold_binding_use(ref_id, decl_block, decl_id, ev);
        }
    }

    fn fold_binding_use(
        &mut self,
        ref_id: NodeId,
        decl_block: Option<NodeId>,
        decl_id: NodeId,
        ev: &mut InstancePropertyEvidence,
    ) {
        let (child_span, use_id) = ast::non_transparent_parent(self.semantic, ref_id);
        match self.semantic.nodes().kind(use_id) {
            AstKind::ThrowStatement(_) => {
                if nearest_function(self.semantic, ref_id)
                    == nearest_function(self.semantic, decl_id)
                {
                    ev.note_throw();
                } else {
                    ev.opaque_mutation = true;
                    ev.positive_through_throws.clear();
                    ev.throw_count = ev.throw_count.saturating_add(1);
                }
            }
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
                    ev.escape();
                }
            }
            AstKind::CallExpression(_) => ev.escape(),
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
        match self.semantic.nodes().parent_kind(member_id) {
            AstKind::UpdateExpression(_) => ev.invalidate_key(key),
            AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::Delete => {
                ev.invalidate_key(key);
            }
            AstKind::AssignmentExpression(assign) if assign.left.span() == member_span => {
                if assign.operator != AssignmentOperator::Assign
                    || !ast::same_unconditional_block(self.semantic, member_id, decl_block)
                {
                    ev.invalidate_key(key);
                    return;
                }
                match key {
                    Some(key) => {
                        let state = self.classify_delay_value(&assign.right);
                        ev.properties.insert(key.to_string(), state);
                    }
                    None => ev.invalidate_key(None),
                }
            }
            _ => {}
        }
    }
}

fn nearest_function(semantic: &oxc_semantic::Semantic<'_>, node: NodeId) -> Option<NodeId> {
    semantic.nodes().ancestors(node).find_map(|ancestor| {
        matches!(
            ancestor.kind(),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        )
        .then_some(ancestor.id())
    })
}
