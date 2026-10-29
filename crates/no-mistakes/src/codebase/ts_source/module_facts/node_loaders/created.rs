use super::*;

impl<'s, 'a> Collector<'s, 'a> {
    /// True for a callee bound by a `const` initialized as
    /// `createRequire(import.meta.url)`, where `createRequire` is an import
    /// (or a member of an imported namespace) from `module` or `node:module`.
    /// Constants and imports cannot be rebound, and the `import.meta.url` base
    /// keeps relative specifiers relative to this module, so calling it is a
    /// `require` load. Aliases, mutable bindings, and other bases are not.
    /// The callee must already have a runtime binding.
    pub(in super::super) fn created_require(&mut self, callee: &IdentifierReference<'a>) -> bool {
        let scoping = self.semantic.scoping();
        let symbol = scoping
            .get_reference(callee.reference_id.get().expect("semantic call reference"))
            .symbol_id()
            .expect("created require callee has a binding");
        if !scoping.symbol_flags(symbol).is_const_variable() {
            return false;
        }
        let nodes = self.semantic.nodes();
        let declaration = scoping.symbol_declaration(symbol);
        let AstKind::VariableDeclarator(VariableDeclarator {
            id: BindingPattern::BindingIdentifier(_),
            init: Some(init),
            ..
        }) = nodes.kind(declaration)
        else {
            return false;
        };
        if matches!(
            nodes.parent_kind(nodes.parent_id(declaration)),
            AstKind::ExportNamedDeclaration(_) | AstKind::ExportDeclaration(_)
        ) {
            return false;
        }
        let Expression::CallExpression(created) = init.get_inner_expression() else {
            return false;
        };
        let [argument] = created.arguments.as_slice() else {
            return false;
        };
        argument
            .as_expression()
            .is_some_and(|argument| is_import_meta_url(argument.get_inner_expression()))
            && matches!(self.origin(&created.callee), Some(Origin::Factory))
            && self.imported_object(&created.callee)
    }

    /// The factory is an import binding or a member of one, never an alias.
    fn imported_object(&self, factory: &Expression<'a>) -> bool {
        let object = member(factory).map_or(factory, |(object, _)| object);
        let scoping = self.semantic.scoping();
        matches!(object.get_inner_expression(), Expression::Identifier(id)
            if scoping
                .get_reference(id.reference_id.get().expect("semantic factory reference"))
                .symbol_id()
                .is_some_and(|symbol| scoping.symbol_flags(symbol).is_import()))
    }
}

fn is_import_meta_url(expression: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(url) = expression else {
        return false;
    };
    url.property.name == "url" && matches!(&url.object, Expression::ImportMeta(_))
}
