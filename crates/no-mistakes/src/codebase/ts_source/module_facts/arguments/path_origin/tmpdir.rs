use super::Projector;
use oxc_ast::{ast::*, AstKind};

impl<'a> Projector<'a, '_> {
    pub(super) fn node_tmpdir(&mut self, callee: &Expression<'a>) -> bool {
        let (id, namespace) = match callee {
            Expression::Identifier(id) => (id, false),
            Expression::StaticMemberExpression(member) if member.property.name == "tmpdir" => {
                let Expression::Identifier(id) = member.object.get_inner_expression() else {
                    return false;
                };
                (id, true)
            }
            _ => return false,
        };
        let Some(symbol) = self.binding_symbol(id) else {
            return false;
        };
        let semantic = self.semantic;
        let declaration = semantic.scoping().symbol_declaration(symbol);
        let imported = match semantic.nodes().kind(declaration) {
            AstKind::ImportSpecifier(binding) if !namespace && !binding.import_kind.is_type() => {
                matches!(&binding.imported, ModuleExportName::IdentifierName(name) if name.name == "tmpdir")
            }
            AstKind::ImportNamespaceSpecifier(_) | AstKind::ImportDefaultSpecifier(_) => namespace,
            AstKind::VariableDeclarator(binding) if self.immutable_binding(symbol) => {
                let matched = match &binding.id {
                    BindingPattern::BindingIdentifier(_) => namespace,
                    BindingPattern::ObjectPattern(pattern) if !namespace => pattern.properties.iter().any(|property| {
                        !property.computed
                            && crate::ast::property_key_name(&property.key).as_deref() == Some("tmpdir")
                            && matches!(&property.value, BindingPattern::BindingIdentifier(id) if id.symbol_id.get() == Some(symbol))
                    }),
                    _ => false,
                };
                return matched
                    && binding
                        .init
                        .as_ref()
                        .is_some_and(|init| self.os_require(init));
            }
            _ => false,
        };
        imported
            && matches!(semantic.nodes().parent_kind(declaration), AstKind::ImportDeclaration(import)
            if !import.import_kind.is_type() && matches!(crate::ast::js_literal(&import.source).as_ref(), "os" | "node:os"))
    }

    fn os_require(&self, expression: &Expression<'_>) -> bool {
        let Expression::CallExpression(call) = expression.get_inner_expression() else {
            return false;
        };
        let Expression::Identifier(loader) = call.callee.get_inner_expression() else {
            return false;
        };
        loader.name == "require"
            && loader.reference_id.get().is_some_and(|reference| {
                self.semantic
                    .scoping()
                    .get_reference(reference)
                    .symbol_id()
                    .is_none()
            })
            && call.arguments.len() == 1
            && matches!(call.arguments.first().and_then(Argument::as_expression).map(Expression::get_inner_expression),
                Some(Expression::StringLiteral(source)) if matches!(crate::ast::js_literal(source).as_ref(), "os" | "node:os"))
    }
}
