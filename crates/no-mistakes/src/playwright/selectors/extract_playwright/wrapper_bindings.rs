use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(super) struct WrapperCall {
    pub(super) call_name: String,
    pub(super) test_id_argument: usize,
}

#[derive(Default)]
pub(super) struct WrapperBindings {
    direct: HashMap<String, WrapperCall>,
    namespaces: HashMap<String, HashMap<String, usize>>,
    ambiguous_direct: HashSet<String>,
    ambiguous_namespaces: HashSet<(String, String)>,
}

impl WrapperBindings {
    pub(super) fn from_program(
        program: &oxc_ast::ast::Program<'_>,
        wrappers: &[crate::config::v2::schema::PlaywrightSelectorWrapper],
        importing_file: Option<&std::path::Path>,
        module_resolution: Option<&crate::codebase::check_facts::PlaywrightModuleResolution>,
    ) -> Self {
        let mut bindings = Self::default();
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            let imported_module = import.source.value.as_str();
            let configured = wrappers
                .iter()
                .filter(|wrapper| match (importing_file, module_resolution) {
                    (Some(importing_file), Some(resolution)) => {
                        resolution.modules_match(&wrapper.module, imported_module, importing_file)
                    }
                    _ => wrapper.module == imported_module,
                })
                .collect::<Vec<_>>();
            if configured.is_empty() {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        let imported = specifier.imported.name();
                        for wrapper in configured
                            .iter()
                            .filter(|wrapper| imported == wrapper.export)
                        {
                            bindings.insert_direct(
                                specifier.local.name.as_str(),
                                WrapperCall {
                                    call_name: specifier.local.name.to_string(),
                                    test_id_argument: wrapper.test_id_argument,
                                },
                            );
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        for wrapper in configured
                            .iter()
                            .filter(|wrapper| wrapper.export == "default")
                        {
                            bindings.insert_direct(
                                specifier.local.name.as_str(),
                                WrapperCall {
                                    call_name: specifier.local.name.to_string(),
                                    test_id_argument: wrapper.test_id_argument,
                                },
                            );
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        for wrapper in &configured {
                            bindings.insert_namespace(
                                specifier.local.name.as_str(),
                                &wrapper.export,
                                wrapper.test_id_argument,
                            );
                        }
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(_) => {}
                }
            }
        }
        bindings
    }

    fn insert_direct(&mut self, local: &str, call: WrapperCall) {
        if self.ambiguous_direct.contains(local) {
            return;
        }
        if self
            .direct
            .get(local)
            .is_some_and(|existing| existing.test_id_argument != call.test_id_argument)
        {
            self.direct.remove(local);
            self.ambiguous_direct.insert(local.to_string());
            return;
        }
        self.direct.insert(local.to_string(), call);
    }

    fn insert_namespace(&mut self, local: &str, export: &str, test_id_argument: usize) {
        let identity = (local.to_string(), export.to_string());
        if self.ambiguous_namespaces.contains(&identity) {
            return;
        }
        let exports = self.namespaces.entry(local.to_string()).or_default();
        if exports
            .get(export)
            .is_some_and(|existing| *existing != test_id_argument)
        {
            exports.remove(export);
            self.ambiguous_namespaces.insert(identity);
            return;
        }
        exports.insert(export.to_string(), test_id_argument);
    }

    pub(super) fn call(
        &self,
        callee: &oxc_ast::ast::Expression<'_>,
        shadow_scopes: &[HashSet<String>],
    ) -> Option<WrapperCall> {
        match callee {
            oxc_ast::ast::Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                (!is_shadowed(name, shadow_scopes))
                    .then(|| self.direct.get(name).cloned())
                    .flatten()
            }
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                let oxc_ast::ast::Expression::Identifier(object) = &member.object else {
                    return None;
                };
                let namespace = object.name.as_str();
                if is_shadowed(namespace, shadow_scopes) {
                    return None;
                }
                self.namespaces
                    .get(namespace)?
                    .get(member.property.name.as_str())
                    .map(|test_id_argument| WrapperCall {
                        call_name: format!("{namespace}.{}", member.property.name),
                        test_id_argument: *test_id_argument,
                    })
            }
            oxc_ast::ast::Expression::ParenthesizedExpression(parenthesized) => {
                self.call(&parenthesized.expression, shadow_scopes)
            }
            _ => None,
        }
    }

    pub(super) fn is_shadowed_configured_call(
        &self,
        callee: &oxc_ast::ast::Expression<'_>,
        shadow_scopes: &[HashSet<String>],
    ) -> bool {
        match callee {
            oxc_ast::ast::Expression::Identifier(identifier) => {
                self.direct.contains_key(identifier.name.as_str())
                    && is_shadowed(identifier.name.as_str(), shadow_scopes)
            }
            oxc_ast::ast::Expression::StaticMemberExpression(member) => {
                let oxc_ast::ast::Expression::Identifier(object) = &member.object else {
                    return false;
                };
                self.namespaces
                    .get(object.name.as_str())
                    .is_some_and(|exports| exports.contains_key(member.property.name.as_str()))
                    && is_shadowed(object.name.as_str(), shadow_scopes)
            }
            oxc_ast::ast::Expression::ParenthesizedExpression(parenthesized) => {
                self.is_shadowed_configured_call(&parenthesized.expression, shadow_scopes)
            }
            _ => false,
        }
    }
}

fn is_shadowed(name: &str, scopes: &[HashSet<String>]) -> bool {
    scopes.iter().rev().any(|scope| scope.contains(name))
}
