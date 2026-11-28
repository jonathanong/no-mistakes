use super::ParameterFields;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

#[derive(Default)]
pub(in crate::codebase::postgres::embedded::walk) struct Writes {
    names: FxHashSet<String>,
    opaque: bool,
    escaped: FxHashSet<String>,
    calls: Vec<oxc_span::Span>,
    suspensions: Vec<oxc_span::Span>,
    reads: std::collections::HashMap<String, std::collections::HashMap<String, u32>>,
}

impl<'a> Visit<'a> for Writes {
    fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
        self.names.insert(id.name.to_string());
    }
    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = target {
            self.names.insert(id.name.to_string());
        }
        if let Some(member) = target.as_member_expression() {
            self.write_member(member);
        }
        walk::walk_assignment_target(self, target);
    }
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        if let Some(expression) = target.get_expression() {
            self.write_expression(expression);
        }
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        property: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.names.insert(property.binding.name.to_string());
        walk::walk_assignment_target_property_identifier(self, property);
    }
    fn visit_update_expression(&mut self, update: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &update.argument {
            self.names.insert(id.name.to_string());
        }
        if let Some(member) = update.argument.as_member_expression() {
            self.write_member(member);
        }
        if let Some(expression) = update.argument.get_expression() {
            self.write_expression(expression);
        }
        walk::walk_update_expression(self, update);
    }
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        self.opaque |= matches!(id.name.as_str(), "eval" | "arguments");
        self.escaped.insert(id.name.to_string());
    }
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Expression::Identifier(root) = unwrap_ts_wrappers(&member.object) {
            self.opaque |= matches!(root.name.as_str(), "eval" | "arguments");
            self.reads
                .entry(root.name.to_string())
                .or_default()
                .entry(member.property.name.to_string())
                .or_insert(member.span.start);
        } else {
            self.visit_expression(&member.object);
        }
    }
    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if unary.operator.is_delete() {
            self.write_expression(&unary.argument);
        }
        walk::walk_unary_expression(self, unary);
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.calls.push(call.span);
        if let Some(member) = unwrap_ts_wrappers(&call.callee).as_member_expression() {
            self.escaped.extend(super::receiver_name(member.object()));
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_tagged_template_expression(&mut self, expression: &TaggedTemplateExpression<'a>) {
        self.calls.push(expression.span);
        if let Some(member) = unwrap_ts_wrappers(&expression.tag).as_member_expression() {
            self.escaped.extend(super::receiver_name(member.object()));
        }
        walk::walk_tagged_template_expression(self, expression);
    }
    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        self.calls.push(expression.span);
        walk::walk_new_expression(self, expression);
    }
    fn visit_await_expression(&mut self, expression: &AwaitExpression<'a>) {
        self.suspensions.push(expression.span);
        self.calls.push(expression.span);
        walk::walk_await_expression(self, expression);
    }
    fn visit_yield_expression(&mut self, expression: &YieldExpression<'a>) {
        self.suspensions.push(expression.span);
        self.calls.push(expression.span);
        walk::walk_yield_expression(self, expression);
    }
}

fn primitive(ty: &TSType<'_>, extra_ok: &impl Fn(&TSType<'_>) -> bool) -> bool {
    match ty {
        TSType::TSParenthesizedType(inner) => primitive(&inner.type_annotation, extra_ok),
        TSType::TSUnionType(union) => union.types.iter().all(|t| primitive(t, extra_ok)),
        TSType::TSLiteralType(literal) => matches!(literal.literal, TSLiteral::StringLiteral(_)),
        TSType::TSStringKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSBooleanKeyword(_) => true,
        other => extra_ok(other),
    }
}

impl Writes {
    fn write_expression(&mut self, expression: &Expression<'_>) {
        let expression = unwrap_ts_wrappers(expression);
        if let Expression::Identifier(root) = expression {
            self.names.insert(root.name.to_string());
        } else if let Some(member) = expression.as_member_expression() {
            self.write_member(member);
        }
    }
    fn write_member(&mut self, member: &MemberExpression<'_>) {
        let mut object = unwrap_ts_wrappers(member.object());
        while let Some(member) = object.as_member_expression() {
            object = unwrap_ts_wrappers(member.object());
        }
        if let Expression::Identifier(root) = object {
            self.names.insert(root.name.to_string());
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn fields(
        &self,
        params: &FormalParameters<'_>,
    ) -> Vec<(u32, ParameterFields)> {
        if self.opaque {
            return Vec::new();
        }
        params
            .items
            .iter()
            .filter_map(|param| {
                let BindingPattern::BindingIdentifier(id) = &param.pattern else {
                    return None;
                };
                if self.names.contains(id.name.as_str()) || self.escaped.contains(id.name.as_str())
                {
                    return None;
                }
                if param.type_annotation.as_ref().is_some_and(|annotation| {
                    matches!(&annotation.type_annotation, TSType::TSTypeLiteral(literal)
                        if literal.members.iter().any(|member| matches!(member,
                            TSSignature::TSMethodSignature(method) if method.kind != TSMethodSignatureKind::Method)))
                }) { return None; }
                let fields = self.reads.get(id.name.as_str())?.clone();
                Some((
                    id.span.start,
                    ParameterFields {
                        fields,
                        calls: self.calls.clone(),
                        suspensions: self.suspensions.clone(),
                    },
                ))
            })
            .collect()
    }
    pub(in crate::codebase::postgres::embedded::walk) fn parameters(
        &self,
        params: &FormalParameters<'_>,
        extra_ok: impl Fn(&TSType<'_>) -> bool,
    ) -> Vec<u32> {
        params
            .items
            .iter()
            .filter_map(|param| {
                let BindingPattern::BindingIdentifier(id) = &param.pattern else {
                    return None;
                };
                let primitive = param
                    .type_annotation
                    .as_ref()
                    .is_some_and(|ty| primitive(&ty.type_annotation, &extra_ok));
                (primitive && !self.opaque && !self.names.contains(id.name.as_str()))
                    .then_some(id.span.start)
            })
            .collect()
    }
}
