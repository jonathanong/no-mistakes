use super::*;

pub(super) enum Source<'a> {
    Known(Origin),
    Alias(&'a Expression<'a>, bool),
}

impl<'a> Collector<'_, 'a> {
    pub(super) fn binding(&self, symbol: SymbolId) -> Option<Source<'a>> {
        let declaration = self.semantic.scoping().symbol_declaration(symbol);
        let kind = self.semantic.nodes().kind(declaration);
        let imported = match kind {
            AstKind::ImportSpecifier(binding) if !binding.import_kind.is_type() => {
                (super::super::modules::name(&binding.imported) == "createRequire")
                    .then_some(Origin::Factory)
            }
            AstKind::ImportNamespaceSpecifier(_) | AstKind::ImportDefaultSpecifier(_) => {
                Some(Origin::ModuleNamespace)
            }
            AstKind::VariableDeclarator(binding) => {
                let initializer = binding.init.as_ref()?;
                return match &binding.id {
                    BindingPattern::BindingIdentifier(id) if id.symbol_id.get() == Some(symbol) => {
                        Some(Source::Alias(initializer, false))
                    }
                    BindingPattern::ObjectPattern(pattern) => {
                        pattern.properties.iter().find_map(|property| {
                            let BindingPattern::BindingIdentifier(id) = &property.value else {
                                return None;
                            };
                            (id.symbol_id.get() == Some(symbol)
                                && !property.computed
                                && crate::ast::property_key_name(&property.key).as_deref()
                                    == Some("createRequire"))
                            .then_some(Source::Alias(initializer, true))
                        })
                    }
                    _ => None,
                };
            }
            _ => None,
        }?;
        match self.semantic.nodes().parent_kind(declaration) {
            AstKind::ImportDeclaration(import)
                if !import.import_kind.is_type()
                    && is_module(&crate::ast::js_literal(&import.source)) =>
            {
                Some(Source::Known(imported))
            }
            _ => None,
        }
    }
}
