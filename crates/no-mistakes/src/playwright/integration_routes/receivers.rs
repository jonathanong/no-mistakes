use crate::playwright::ast;
use oxc_ast::ast::{
    AssignmentExpression, BindingIdentifier, Expression, Program, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::{BTreeMap, BTreeSet};

/// Ambiguous/shadowed receiver names are intentionally rejected rather than
/// conflating lexical bindings. A receiver must have a concrete constructor.
#[derive(Default)]
pub(super) struct Receivers {
    declarations: BTreeMap<String, usize>,
    constructors: BTreeMap<String, String>,
    invalid: BTreeSet<String>,
    runner_imports: BTreeMap<String, String>,
    constructor_origins: BTreeMap<String, BTreeSet<String>>,
    constructor_scopes: BTreeMap<String, BTreeSet<RegistrationScope>>,
    declaration_scopes: BTreeMap<String, RegistrationScope>,
    scope: RegistrationScope,
    origins: Vec<String>,
    initialization_context: bool,
    registration_context: bool,
}

include!("receivers_initialization.rs");
include!("receivers_mutation.rs");
include!("receivers_scope.rs");

impl Receivers {
    pub(super) fn collect(
        program: &Program<'_>,
        imports: &[crate::codebase::dependencies::extract::ImportedBinding],
    ) -> Self {
        let mut census = Self {
            initialization_context: true,
            registration_context: true,
            runner_imports: imports
                .iter()
                .filter(|binding| !binding.is_type_only && binding.specifier == "vitest")
                .map(|binding| (binding.local.clone(), binding.imported.clone()))
                .collect(),
            ..Default::default()
        };
        census.visit_program(program);
        census
    }

    pub(super) fn unique(&self, name: &str) -> bool {
        self.declarations.get(name) == Some(&1) && !self.invalid.contains(name)
    }

    pub(super) fn constructor(&self, name: &str, scope: &RegistrationScope) -> Option<&str> {
        self.unique(name)
            .then(|| self.constructors.get(name).map(String::as_str))
            .flatten()
            .filter(|constructor| self.unique(constructor))
            .filter(|_| {
                self.constructor_origins
                    .get(name)
                    .is_none_or(|origins| origins.iter().all(|origin| self.unique(origin)))
            })
            .filter(|_| {
                self.declaration_scopes
                    .get(name)
                    .is_some_and(|declaration| {
                        declaration.hook.is_none()
                            && declaration.applies_to(scope)
                            && self
                                .constructor_scopes
                                .get(name)
                                .is_some_and(|initializers| {
                                    initializers.iter().any(|initializer| {
                                        initializer.applies_to(scope)
                                            && declaration.applies_to(initializer)
                                            && (initializer.test.is_none()
                                                || initializer == declaration)
                                    })
                                })
                    })
            })
    }

    fn assign(&mut self, name: &str, value: Option<&Expression<'_>>) {
        if value.is_some() && !self.initialization_context {
            self.invalid.insert(name.to_string());
            return;
        }
        let constructor = value.and_then(|value| match value {
            Expression::NewExpression(new) => ast::expression_path(&new.callee)
                .filter(|path| path.len() == 1)
                .map(|path| path[0].clone()),
            _ => None,
        });
        let Some(constructor) = constructor else {
            if value.is_some() {
                self.invalid.insert(name.to_string());
            }
            return;
        };
        if self
            .constructors
            .get(name)
            .is_some_and(|existing| existing != &constructor)
        {
            self.invalid.insert(name.to_string());
        }
        self.constructors.insert(name.to_string(), constructor);
        self.constructor_origins
            .entry(name.to_string())
            .or_default()
            .extend(self.origins.iter().cloned());
        self.constructor_scopes
            .entry(name.to_string())
            .or_default()
            .insert(self.scope.clone());
    }
}

include!("receivers_visit.rs");

impl Receivers {
    fn invalidate_object(&mut self, object: &Expression<'_>) {
        if let Some(path) = ast::expression_path(object) {
            if let Some(binding) = path.first() {
                self.invalid.insert(binding.clone());
            }
        }
    }
}
