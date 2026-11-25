use super::Bindings;
use crate::codebase::rules::test_reliability::playwright_provenance::ImportRef;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Class, ClassElement, Expression, FormalParameter, FormalParameters};
use oxc_ast::AstKind;
use oxc_semantic::SymbolId;
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

impl<'ast> Bindings<'_, 'ast> {
    pub(super) fn possible_binding_values(&self, symbol: SymbolId) -> Vec<&'ast Expression<'ast>> {
        let nodes = self.semantic.nodes();
        let scope = self.semantic.scoping();
        let declaration = scope.symbol_declaration(symbol);
        let mut values = Vec::new();
        match nodes.kind(declaration) {
            AstKind::VariableDeclarator(_) => {
                if let AstKind::ForOfStatement(loop_node) = nodes.parent_kind(nodes.parent_id(declaration)) {
                    values.push(&loop_node.right);
                }
            }
            AstKind::FormalParameter(parameter) => {
                values.extend(parameter.initializer.as_deref());
                values.extend(self.parameter_arguments(parameter));
            }
            _ => {}
        }
        values.extend(scope.get_resolved_references(symbol).filter_map(|reference| {
            if !reference.flags().is_write() {
                return None;
            }
            match nodes.parent_kind(reference.node_id()) {
                AstKind::AssignmentExpression(assignment)
                    if matches!(
                        assignment.operator,
                        AssignmentOperator::Assign
                            | AssignmentOperator::LogicalNullish
                            | AssignmentOperator::LogicalOr
                            | AssignmentOperator::LogicalAnd
                    ) =>
                {
                    Some(&assignment.right)
                }
                _ => None,
            }
        }));
        values
    }

    fn parameter_owner(
        &self,
        parameter: &FormalParameter<'_>,
    ) -> Option<(SymbolId, &'ast FormalParameters<'ast>)> {
        let nodes = self.semantic.nodes();
        let owner = nodes.parent_id(nodes.parent_id(parameter.node_id.get()));
        match nodes.kind(owner) {
            AstKind::Function(func) if func.is_declaration() => {
                Some((func.id.as_ref()?.symbol_id.get()?, &func.params))
            }
            AstKind::Function(func) => {
                Some((self.variable_owner(owner)?, &func.params))
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                Some((self.variable_owner(owner)?, &arrow.params))
            }
            _ => None,
        }
    }

    fn variable_owner(&self, node: oxc_syntax::node::NodeId) -> Option<SymbolId> {
        let declaration = self.semantic.nodes().parent_kind(node).as_variable_declarator()?;
        declaration.id.get_binding_identifier()?.symbol_id.get()
    }

    fn parameter_arguments(&self, parameter: &FormalParameter<'_>) -> Vec<&'ast Expression<'ast>> {
        let Some((function, params)) = self.parameter_owner(parameter) else {
            return Vec::new();
        };
        let Some(index) = params.items.iter().position(|item| item.node_id.get() == parameter.node_id.get()) else {
            return Vec::new();
        };
        let nodes = self.semantic.nodes();
        self.semantic.scoping().get_resolved_references(function).filter_map(|reference| {
            let AstKind::CallExpression(call) = nodes.parent_kind(reference.node_id()) else {
                return None;
            };
            if call.callee.span() != nodes.kind(reference.node_id()).span() {
                return None;
            }
            let mut args = call.arguments.iter();
            if args.by_ref().take(index).any(|argument| argument.is_spread()) {
                return None;
            }
            args.next()?.as_expression()
        }).collect()
    }

    pub(super) fn receiver_class(&self, object: &Expression<'_>) -> Option<&'ast Class<'ast>> {
        let nodes = self.semantic.nodes();
        match object {
            Expression::NewExpression(instance) => {
                let Expression::Identifier(name) = unwrap_ts_wrappers(&instance.callee) else {
                    return None;
                };
                let symbol = self.symbol(name)?;
                match nodes.kind(self.semantic.scoping().symbol_declaration(symbol)) {
                    AstKind::Class(class) => Some(class),
                    AstKind::VariableDeclarator(decl) => {
                        let Expression::ClassExpression(class) = self.constant_initializer(name)? else {
                            return None;
                        };
                        decl.id.is_binding_identifier().then_some(class)
                    }
                    _ => None,
                }
            }
            Expression::ThisExpression(this) => {
                for node in nodes.ancestors(this.node_id.get()) {
                    match node.kind() {
                        AstKind::Class(class) => return Some(class),
                        AstKind::Function(_) if !matches!(nodes.parent_kind(node.id()), AstKind::MethodDefinition(_)) => return None,
                        _ => {}
                    }
                }
                None
            }
            _ => None,
        }
    }
    pub(in crate::codebase::rules::test_reliability::ast) fn namespace_member_origin(
        &self,
        object: &Expression<'_>,
        property: &str,
    ) -> Option<ImportRef> {
        let mut current = unwrap_ts_wrappers(object);
        for _ in 0..8 {
            let Expression::Identifier(identifier) = current else {
                break;
            };
            let symbol = self.symbol(identifier)?;
            if let Some(import) = self.import_origin(symbol, Some(property)) {
                return Some(import);
            }
            let Some(init) = self.constant_initializer(identifier) else {
                break;
            };
            current = unwrap_ts_wrappers(init);
        }
        None
    }
}

pub(super) fn class_field_values<'ast, 'name>(
    class: &'ast Class<'ast>,
    property: Option<&'name str>,
) -> impl Iterator<Item = &'ast Expression<'ast>> + use<'ast, 'name> {
    class.body.body.iter().filter_map(move |element| {
        let ClassElement::PropertyDefinition(field) = element else {
            return None;
        };
        let matches = match property {
            Some(property) => field.key.static_name().is_some_and(|name| name == property),
            None => true,
        };
        (!field.r#static && matches)
            .then_some(field.value.as_ref())
            .flatten()
    })
}
