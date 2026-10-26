use super::const_values::{self, record_unknown, try_record_resolved};
use super::patterns::single_binding_name;
use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    BindingPattern, Class, Declaration, Expression, FormalParameters, Function, FunctionBody,
    Statement, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;

mod statements;

impl<'a> NextjsCachingVisitor<'a> {
    pub(crate) fn push_const_scope(&mut self) {
        const_values::push_scope(&mut self.const_scopes);
    }

    pub(crate) fn pop_const_scope(&mut self) {
        const_values::pop_scope(&mut self.const_scopes);
    }

    pub(crate) fn open_function_scope(
        &mut self,
        params: &FormalParameters<'a>,
        name: Option<&str>,
    ) {
        self.push_const_scope();
        if let Some(name) = name {
            record_unknown(&mut self.const_scopes, name);
        }
        self.record_formal_parameters(params);
    }

    pub(crate) fn open_function_body_scope(&mut self, body: &FunctionBody<'a>) {
        // Body bindings are not visible to default parameter initializers.
        self.push_const_scope();
        self.precollect_hoisted_vars(&body.statements);
        self.precollect_statements(&body.statements);
    }

    pub(crate) fn precollect_statements<'b>(
        &mut self,
        statements: impl IntoIterator<Item = &'b Statement<'a>>,
    ) where
        'a: 'b,
    {
        let mut declarations = Vec::new();
        for statement in statements {
            // These are lexical statement bodies; module exports are collected separately.
            match statement.as_declaration() {
                Some(Declaration::VariableDeclaration(declaration)) => {
                    declarations.push(declaration.as_ref());
                }
                Some(Declaration::FunctionDeclaration(function)) => {
                    self.record_name(function.id.as_ref().map(|id| id.name.as_str()));
                }
                Some(Declaration::ClassDeclaration(class)) => {
                    self.record_name(class.id.as_ref().map(|id| id.name.as_str()));
                }
                _ => {}
            }
        }
        // Reserve every name before resolving aliases, including later declarations.
        for declaration in &declarations {
            self.record_bound_names(declaration);
        }
        let mut pending: Vec<&VariableDeclarator<'a>> = declarations
            .iter()
            .filter(|declaration| declaration.kind == VariableDeclarationKind::Const)
            .flat_map(|declaration| declaration.declarations.iter())
            .collect();
        loop {
            let before = pending.len();
            pending.retain(|declarator| !self.try_record_declarator(declarator));
            if pending.len() == before {
                break;
            }
        }
        for declarator in pending {
            self.record_pattern_unknown(&declarator.id);
        }
    }

    pub(crate) fn record_name(&mut self, name: Option<&str>) {
        if let Some(name) = name {
            record_unknown(&mut self.const_scopes, name);
        }
    }

    pub(crate) fn precollect_hoisted_vars(&mut self, statements: &[Statement<'a>]) {
        // Scan declaration-bearing statement structure only, stopping at new var owners.
        let mut collector = HoistedVars(self);
        for statement in statements {
            collector.visit_statement(statement);
        }
    }

    pub(crate) fn open_loop_scope(&mut self, declaration: Option<&VariableDeclaration<'a>>) {
        self.push_const_scope();
        if let Some(declaration) = declaration {
            self.record_bound_names(declaration);
        }
    }

    pub(crate) fn record_const_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Const {
            for declarator in &declaration.declarations {
                if !self.try_record_declarator(declarator) {
                    self.record_pattern_unknown(&declarator.id);
                }
            }
            return;
        }
        self.record_bound_names(declaration);
    }

    fn record_bound_names(&mut self, declaration: &VariableDeclaration<'a>) {
        for declarator in &declaration.declarations {
            self.record_pattern_unknown(&declarator.id);
        }
    }

    fn try_record_declarator(&mut self, declarator: &VariableDeclarator<'a>) -> bool {
        let (Some(name), Some(init)) = (
            single_binding_name(&declarator.id),
            declarator.init.as_ref(),
        ) else {
            return false;
        };
        try_record_resolved(&mut self.const_scopes, &name, init)
    }

    fn record_formal_parameters(&mut self, params: &FormalParameters<'a>) {
        for item in &params.items {
            self.record_pattern_unknown(&item.pattern);
        }
        if let Some(rest) = &params.rest {
            self.record_pattern_unknown(&rest.rest.argument);
        }
    }

    pub(crate) fn record_pattern_unknown(&mut self, pattern: &BindingPattern<'a>) {
        match pattern {
            BindingPattern::BindingIdentifier(identifier) => {
                record_unknown(&mut self.const_scopes, identifier.name.as_str());
            }
            BindingPattern::ObjectPattern(pattern) => {
                for property in &pattern.properties {
                    self.record_pattern_unknown(&property.value);
                }
                if let Some(rest) = &pattern.rest {
                    self.record_pattern_unknown(&rest.argument);
                }
            }
            BindingPattern::ArrayPattern(pattern) => {
                for element in pattern.elements.iter().flatten() {
                    self.record_pattern_unknown(element);
                }
                if let Some(rest) = &pattern.rest {
                    self.record_pattern_unknown(&rest.argument);
                }
            }
            BindingPattern::AssignmentPattern(pattern) => {
                self.record_pattern_unknown(&pattern.left);
            }
        }
    }
}

struct HoistedVars<'v, 'a>(&'v mut NextjsCachingVisitor<'a>);

impl<'a> Visit<'a> for HoistedVars<'_, 'a> {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            self.0.record_bound_names(declaration);
        }
    }

    fn visit_expression(&mut self, _: &Expression<'a>) {}

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_class(&mut self, _: &Class<'a>) {}
}
