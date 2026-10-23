use super::const_values::{self, record_unknown, try_record_resolved};
use super::patterns::single_binding_name;
use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    BindingPattern, Declaration, FormalParameters, Statement, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarator,
};

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
        statements: Option<&[Statement<'a>]>,
    ) {
        self.push_const_scope();
        self.record_formal_parameters(params);
        if let Some(statements) = statements {
            self.precollect_statements(statements);
        }
    }

    pub(crate) fn precollect_statements(&mut self, statements: &[Statement<'a>]) {
        let declarations = direct_declarations(statements);
        for declaration in &declarations {
            if declaration.kind != VariableDeclarationKind::Const {
                self.record_bound_names(declaration);
            }
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

    fn record_pattern_unknown(&mut self, pattern: &BindingPattern<'a>) {
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

fn direct_declarations<'a, 'b>(
    statements: &'b [Statement<'a>],
) -> Vec<&'b VariableDeclaration<'a>> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::VariableDeclaration(declaration) => Some(declaration.as_ref()),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => Some(declaration.as_ref()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}
