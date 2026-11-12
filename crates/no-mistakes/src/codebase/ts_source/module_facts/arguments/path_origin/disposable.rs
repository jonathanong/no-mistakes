use super::*;

impl<'a> Projector<'a, '_> {
    pub(super) fn awaited_disposable_origin(
        &mut self,
        awaited: &AwaitExpression<'a>,
        key: &str,
    ) -> Option<PathOrigin> {
        if key != "path" {
            return None;
        }
        let Expression::CallExpression(call) = awaited.argument.get_inner_expression() else {
            return None;
        };
        self.disposable_factory(call, true)
            .then(|| super::super::composite::disposable_origin(call))
    }

    pub(in super::super) fn disposable_factory(
        &mut self,
        call: &CallExpression<'a>,
        asynchronous: bool,
    ) -> bool {
        let (identifier, member) = match call.callee.get_inner_expression() {
            Expression::Identifier(identifier) => (identifier.as_ref(), None),
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(identifier) = member.object.get_inner_expression()
                else {
                    return false;
                };
                (identifier.as_ref(), Some(member.property.name.as_str()))
            }
            _ => return false,
        };
        let Some(symbol) = self.binding_symbol(identifier) else {
            return false;
        };
        let declaration = self.semantic.scoping().symbol_declaration(symbol);
        let expected = if asynchronous {
            "mkdtempDisposable"
        } else {
            "mkdtempDisposableSync"
        };
        let matches = match self.semantic.nodes().kind(declaration) {
            AstKind::ImportSpecifier(binding) => {
                !binding.import_kind.is_type()
                    && member.is_none()
                    && crate::codebase::ts_source::module_facts::modules::name(&binding.imported)
                        == expected
            }
            AstKind::ImportNamespaceSpecifier(_) => {
                member == Some(expected) && self.disposable_namespace_stable(symbol, expected)
            }
            AstKind::ImportDefaultSpecifier(_) => {
                let module = if asynchronous { "fs/promises" } else { "fs" };
                member == Some(expected) && self.disposable_defaults_stable(module, expected)
            }
            _ => false,
        };
        if !matches {
            return false;
        }
        matches!(self.semantic.nodes().parent_kind(declaration),
        AstKind::ImportDeclaration(import) if !import.import_kind.is_type()
            && if asynchronous {
                matches!(crate::ast::js_literal(&import.source).as_ref(), "node:fs/promises" | "fs/promises")
            } else {
                matches!(crate::ast::js_literal(&import.source).as_ref(), "node:fs" | "fs")
            })
    }
    fn disposable_defaults_stable(&mut self, module: &str, expected: &str) -> bool {
        let semantic = self.semantic;
        for alias in semantic.scoping().symbol_ids() {
            if !self.take() {
                return false;
            }
            let declaration = semantic.scoping().symbol_declaration(alias);
            if !matches!(
                semantic.nodes().kind(declaration),
                AstKind::ImportDefaultSpecifier(_)
            ) {
                continue;
            }
            if matches!(semantic.nodes().parent_kind(declaration),
            AstKind::ImportDeclaration(import)
                if {
                    let source = crate::ast::js_literal(&import.source);
                    source.strip_prefix("node:").unwrap_or(&source) == module
                })
                && !self.disposable_namespace_stable(alias, expected)
            {
                return false;
            }
        }
        true
    }
    fn disposable_namespace_stable(
        &mut self,
        symbol: oxc_semantic::SymbolId,
        expected: &str,
    ) -> bool {
        let semantic = self.semantic;
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return false;
            }
            let member = semantic.nodes().parent_id(reference.node_id());
            let AstKind::StaticMemberExpression(access) = semantic.nodes().kind(member) else {
                return false;
            };
            if access.property.name != expected {
                return false;
            }
            if !matches!(semantic.nodes().parent_kind(member),
                AstKind::CallExpression(call) if call.callee.get_inner_expression().span() == access.span)
            {
                return false;
            }
        }
        true
    }
}
