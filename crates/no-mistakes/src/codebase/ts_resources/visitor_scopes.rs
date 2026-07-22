use super::bindings::binding_names;
use super::{ResourceDiagnostic, ResourceDiagnosticKind, ResourceVisitor};
use oxc_ast::ast::{
    BindingPattern, Declaration, FormalParameters, Function, Statement, VariableDeclaration,
    VariableDeclarationKind,
};

impl<'a> ResourceVisitor<'a> {
    pub(super) fn current_scope(&self) -> Option<String> {
        self.function_stack.last().cloned()
    }

    pub(super) fn emit_diagnostic(&mut self, kind: ResourceDiagnosticKind, offset: u32) {
        self.facts.diagnostics.push(ResourceDiagnostic {
            kind,
            line: crate::codebase::ts_source::byte_offset_to_line(self.source, offset as usize)
                as usize,
            function_scope: self.current_scope(),
        });
    }

    pub(super) fn push_function(&mut self, name: Option<String>) {
        self.anonymous_scopes += usize::from(name.is_none());
        let name = name.unwrap_or_else(|| format!("<anonymous:{}>", self.anonymous_scopes));
        let scope = self
            .function_stack
            .last()
            .map(|parent| format!("{parent}/{name}"))
            .unwrap_or(name);
        self.function_stack.push(scope);
        self.bindings.push(Default::default());
        self.function_binding_scopes.push(self.bindings.len() - 1);
    }

    pub(super) fn pop_function(&mut self) {
        self.function_stack.pop();
        self.bindings.pop();
        self.function_binding_scopes.pop();
    }

    pub(super) fn push_lexical_scope(&mut self) {
        self.bindings.push(Default::default());
    }

    pub(super) fn pop_lexical_scope(&mut self) {
        self.bindings.pop();
    }

    pub(super) fn shadow_pattern(&mut self, pattern: &BindingPattern<'_>) {
        for name in binding_names(pattern) {
            self.declare_binding(&name, None);
        }
    }

    pub(super) fn shadow_parameters(&mut self, params: &FormalParameters<'_>) {
        for parameter in &params.items {
            self.shadow_pattern(&parameter.pattern);
        }
    }

    pub(super) fn shadow_pattern_as_var(&mut self, pattern: &BindingPattern<'_>) {
        for name in binding_names(pattern) {
            self.declare_var_binding(&name, None);
        }
    }

    pub(super) fn predeclare_statement_bindings(&mut self, statements: &[Statement<'_>]) {
        for statement in statements {
            match statement {
                Statement::VariableDeclaration(declaration) => {
                    self.predeclare_variable_declaration(declaration)
                }
                Statement::FunctionDeclaration(function) => self.predeclare_function(function),
                Statement::ClassDeclaration(class) => self.predeclare_class(class),
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = &export.declaration {
                        self.predeclare_declaration(declaration);
                    }
                }
                _ => {}
            }
        }
    }

    fn predeclare_variable_declaration(&mut self, declaration: &VariableDeclaration<'_>) {
        for declarator in &declaration.declarations {
            if declaration.kind == VariableDeclarationKind::Var {
                self.shadow_pattern_as_var(&declarator.id);
            } else {
                self.shadow_pattern(&declarator.id);
            }
        }
    }

    fn predeclare_function(&mut self, function: &Function<'_>) {
        if let Some(id) = &function.id {
            self.declare_binding(id.name.as_str(), None);
        }
    }

    fn predeclare_class(&mut self, class: &oxc_ast::ast::Class<'_>) {
        if let Some(id) = &class.id {
            self.declare_binding(id.name.as_str(), None);
        }
    }

    pub(super) fn predeclare_declaration(&mut self, declaration: &Declaration<'_>) {
        match declaration {
            Declaration::VariableDeclaration(declaration) => {
                self.predeclare_variable_declaration(declaration)
            }
            Declaration::FunctionDeclaration(function) => self.predeclare_function(function),
            Declaration::ClassDeclaration(class) => self.predeclare_class(class),
            _ => {}
        }
    }
}
