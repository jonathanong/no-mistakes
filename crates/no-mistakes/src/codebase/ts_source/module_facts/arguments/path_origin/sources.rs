//! Path origins that start outside the expression: env vars, parameters, getters.
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

/// True for the unshadowed global `process`.
pub(in super::super) fn process_global(
    semantic: &Semantic<'_>,
    expression: &Expression<'_>,
) -> bool {
    let Expression::Identifier(process) = expression.get_inner_expression() else {
        return false;
    };
    process.name == "process"
        && process.reference_id.get().is_some_and(|reference| {
            semantic
                .scoping()
                .get_reference(reference)
                .symbol_id()
                .is_none()
        })
}

/// True for the unshadowed global `process.env`.
pub(in super::super) fn is_process_env(
    semantic: &Semantic<'_>,
    expression: &Expression<'_>,
) -> bool {
    let Expression::StaticMemberExpression(member) = expression.get_inner_expression() else {
        return false;
    };
    member.property.name == "env" && process_global(semantic, &member.object)
}

/// The variable named by `process.env.NAME` or `process.env["NAME"]`.
pub(in super::super) fn env_name(
    semantic: &Semantic<'_>,
    expression: &Expression<'_>,
) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StaticMemberExpression(member) => {
            is_process_env(semantic, &member.object).then(|| member.property.name.to_string())
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::StringLiteral(key) = member.expression.get_inner_expression() else {
                return None;
            };
            is_process_env(semantic, &member.object)
                .then(|| crate::ast::js_literal(key).into_owned())
        }
        _ => None,
    }
}

impl<'a> Projector<'a, '_> {
    pub(super) fn env_origin(&mut self, expression: &Expression<'a>) -> PathOrigin {
        match env_name(self.semantic, expression) {
            Some(name) => PathOrigin {
                env: Some(name),
                ..PathOrigin::unknown()
            },
            None => PathOrigin::unknown(),
        }
    }

    /// A never-reassigned parameter, including one direct object-pattern property.
    pub(super) fn param_origin(&mut self, id: &IdentifierReference<'a>) -> Option<PathOrigin> {
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        if semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|reference| reference.flags().is_write())
        {
            return None;
        }
        // oxc records the formal parameter as the symbol declaration, not the
        // inner binding identifier, so the property is found by scanning that pattern.
        let mut node = semantic.scoping().symbol_declaration(symbol);
        let mut located = None;
        for _ in 0..8 {
            if let AstKind::FormalParameter(parameter) = semantic.nodes().kind(node) {
                let member = match &parameter.pattern {
                    BindingPattern::BindingIdentifier(binding)
                        if binding.symbol_id.get() == Some(symbol) =>
                    {
                        None
                    }
                    BindingPattern::ObjectPattern(pattern) => {
                        Some(super::patterns::direct_property_member(pattern, symbol)?)
                    }
                    _ => return None,
                };
                located = Some((parameter.span, member, node));
                break;
            }
            let parent = semantic.nodes().parent_id(node);
            if parent == node {
                break;
            }
            node = parent;
        }
        let (parameter_span, member, parameter_node) = located?;
        let list = semantic.nodes().parent_node(parameter_node);
        let list_kind = list.kind().as_formal_parameters()?;
        let index = list_kind
            .items
            .iter()
            .position(|item| item.span == parameter_span)?;
        let owner = semantic.nodes().parent_id(list.id());
        let start = semantic.nodes().kind(owner).span().start;
        Some(PathOrigin {
            param: Some(index),
            param_member: member,
            param_owner: Some(start),
            owner_line: Some(super::super::line_in(semantic.source_text(), start)),
            ..PathOrigin::unknown()
        })
    }

    /// 1-based line of a write to a simple parameter, when the binding is one.
    pub(super) fn reassigned_parameter(&mut self, id: &IdentifierReference<'a>) -> Option<u32> {
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        let mut declaration = semantic.scoping().symbol_declaration(symbol);
        let mut parameter = None;
        for _ in 0..3 {
            if let AstKind::FormalParameter(found) = semantic.nodes().kind(declaration) {
                parameter = Some(found);
                break;
            }
            declaration = semantic.nodes().parent_id(declaration);
        }
        let parameter = parameter?;
        if !matches!(parameter.pattern, BindingPattern::BindingIdentifier(_)) {
            return None;
        }
        semantic
            .scoping()
            .get_resolved_references(symbol)
            .find(|reference| reference.flags().is_write())
            .map(|reference| {
                super::super::line_in(
                    semantic.source_text(),
                    semantic.nodes().kind(reference.node_id()).span().start,
                )
            })
    }
}
