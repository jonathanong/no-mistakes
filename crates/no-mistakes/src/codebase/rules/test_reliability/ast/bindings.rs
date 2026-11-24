use super::super::playwright_provenance::ImportRef;
use crate::ast::{js_literal, module_export_name};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Expression, IdentifierReference, ImportDeclarationSpecifier, Program, Statement,
};
use oxc_semantic::{Semantic, SymbolId};
use rustc_hash::FxHashMap;
use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Runner {
    Vitest,
    Playwright,
    Jest,
}

pub(super) struct Bindings<'scope, 'ast> {
    pub(super) semantic: &'scope Semantic<'ast>,
    imports: FxHashMap<SymbolId, (Runner, Option<Cow<'ast, str>>)>,
    other_imports: FxHashMap<SymbolId, ImportRef>,
    vitest_globals: bool,
}

impl<'scope, 'ast> Bindings<'scope, 'ast> {
    pub(super) fn new(
        program: &'ast Program<'ast>,
        semantic: &'scope Semantic<'ast>,
        vitest_globals: bool,
        collect_fixture_imports: bool,
    ) -> Self {
        let mut imports = FxHashMap::default();
        let mut other_imports = FxHashMap::default();
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            let source = js_literal(&import.source);
            let runner = match source.as_ref() {
                "vitest" | "vitest/config" | "vite" => Some(Runner::Vitest),
                "@playwright/test" => Some(Runner::Playwright),
                "@jest/globals" => Some(Runner::Jest),
                _ => None,
            };
            if runner.is_none() && !collect_fixture_imports {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let (local, name) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(binding)
                        if !binding.import_kind.is_type() =>
                    {
                        (&binding.local, Some(module_export_name(&binding.imported)))
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(binding) => {
                        (&binding.local, None)
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(binding)
                        if runner.is_none() =>
                    {
                        (&binding.local, Some(Cow::Borrowed("default")))
                    }
                    _ => continue,
                };
                if let Some(symbol) = local.symbol_id.get() {
                    if let Some(runner) = runner {
                        imports.insert(symbol, (runner, name));
                    } else {
                        other_imports.insert(
                            symbol,
                            ImportRef::new(&source, name.as_deref().unwrap_or("*")),
                        );
                    }
                }
            }
        }
        Self {
            semantic,
            imports,
            other_imports,
            vitest_globals,
        }
    }

    pub(super) fn symbol(&self, identifier: &IdentifierReference<'_>) -> Option<SymbolId> {
        self.semantic
            .scoping()
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()
    }

    pub(super) fn constant_initializer(
        &self,
        identifier: &IdentifierReference<'_>,
    ) -> Option<&'ast Expression<'ast>> {
        let scope = self.semantic.scoping();
        let symbol = self.symbol(identifier)?;
        if !scope.symbol_flags(symbol).is_const_variable() {
            return None;
        }
        let declaration = self
            .semantic
            .nodes()
            .kind(scope.symbol_declaration(symbol))
            .as_variable_declarator()?;
        if declaration.span.end > identifier.span.start || !declaration.id.is_binding_identifier() {
            return None;
        }
        declaration.init.as_ref()
    }

    pub(super) fn any_var_initializer(&self, symbol: SymbolId) -> Option<&'ast Expression<'ast>> {
        self.semantic
            .nodes()
            .kind(self.semantic.scoping().symbol_declaration(symbol))
            .as_variable_declarator()?
            .init
            .as_ref()
    }

    pub(super) fn is_namespace_import(&self, symbol: SymbolId) -> bool {
        self.other_imports
            .get(&symbol)
            .is_some_and(|origin| origin.name.as_ref() == "*")
            || matches!(self.imports.get(&symbol), Some((Runner::Playwright, None)))
    }

    pub(super) fn import_origin(
        &self,
        symbol: SymbolId,
        namespace_member: Option<&str>,
    ) -> Option<ImportRef> {
        if let Some((Runner::Playwright, name)) = self.imports.get(&symbol) {
            return Some(ImportRef::new(
                "@playwright/test",
                match (name.as_deref(), namespace_member) {
                    (Some(name), None) | (None, Some(name)) => name,
                    _ => return None,
                },
            ));
        }
        let origin = self.other_imports.get(&symbol)?;
        if origin.name.as_ref() == "*" {
            Some(ImportRef::new(&origin.specifier, namespace_member?))
        } else {
            namespace_member.is_none().then(|| origin.clone())
        }
    }

    pub(super) fn call<'b>(
        &'b self,
        expression: &'b Expression<'_>,
    ) -> Option<(Runner, Vec<&'b str>, bool)> {
        let mut chain = Vec::new();
        let mut node = unwrap_ts_wrappers(expression);
        while let Some(member) = node.as_member_expression() {
            let name = member.static_property_name()?.as_str()?;
            chain.push(name);
            node = unwrap_ts_wrappers(member.object());
        }
        let Expression::Identifier(identifier) = node else {
            return None;
        };
        let symbol = self.symbol(identifier);
        let (runner, name) = if let Some(symbol) = symbol {
            let (runner, name) = self.imports.get(&symbol)?;
            (*runner, name.as_deref())
        } else {
            let name = identifier.name.as_str();
            match name {
                "jest" => (Runner::Jest, Some(name)),
                "test" | "it" | "describe" if self.vitest_globals => (Runner::Vitest, Some(name)),
                _ => return None,
            }
        };
        chain.extend(name);
        chain.reverse();
        Some((runner, chain, runner == Runner::Vitest && symbol.is_none()))
    }
}
