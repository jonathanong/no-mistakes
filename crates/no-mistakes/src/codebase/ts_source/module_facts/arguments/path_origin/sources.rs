//! Path origins that start outside the expression: env vars, parameters, getters.
use super::{PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;

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

    /// A never-reassigned parameter of the function that contains the reference.
    pub(super) fn param_origin(&mut self, id: &IdentifierReference<'a>) -> Option<PathOrigin> {
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
        if !matches!(parameter.pattern, BindingPattern::BindingIdentifier(_))
            || semantic
                .scoping()
                .get_resolved_references(symbol)
                .any(|reference| reference.flags().is_write())
        {
            return None;
        }
        let list = semantic.nodes().parent_node(declaration);
        let list_kind = list.kind().as_formal_parameters()?;
        let index = list_kind
            .items
            .iter()
            .position(|item| item.span == parameter.span)?;
        let owner = semantic.nodes().parent_id(list.id());
        let reference = semantic.scoping().get_reference(id.reference_id.get()?);
        let innermost = semantic
            .nodes()
            .ancestors(reference.node_id())
            .find(|node| {
                matches!(
                    node.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
            })?;
        (innermost.id() == owner).then(|| PathOrigin {
            param: Some(index),
            ..PathOrigin::unknown()
        })
    }
}
