use super::types::{PlaywrightSelector, SelectorRegexes};
use crate::playwright::playwright_tests;
use oxc_ast::ast::{ForStatement, ForStatementInit, ForStatementLeft, SwitchStatement};
use oxc_ast_visit::walk;
use std::collections::HashSet;

mod entry;
mod shadow_bindings;
mod visitor_calls;
mod visitor_state;
mod wrapper_bindings;
pub(crate) use entry::extract_playwright_selector_occurrences_and_wrapper_calls_from_program;
pub use entry::extract_playwright_selector_occurrences_from_program;
use shadow_bindings::{
    collect_binding_names, collect_direct_lexical_declarations,
    collect_function_scope_declarations, collect_lexical_variable_names,
};
use wrapper_bindings::WrapperBindings;

struct PlaywrightSelectorVisitor<'a, 'r> {
    source: &'a str,
    regexes: &'r SelectorRegexes,
    test_id_attributes: &'r [String],
    status: playwright_tests::TestStatus,
    annotation_status: playwright_tests::TestStatus,
    selectors: Vec<playwright_tests::TestOccurrence<PlaywrightSelector>>,
    current_test_name: Option<String>,
    current_scope: playwright_tests::TestOccurrenceScope,
    describe_stack: Vec<String>,
    wrapper_bindings: WrapperBindings,
    wrapper_call_offsets: HashSet<u32>,
    shadow_scopes: Vec<HashSet<String>>,
}

impl<'a> oxc_ast_visit::Visit<'a> for PlaywrightSelectorVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.extract_call_selectors(call);
        self.traverse_call(call);
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_statement(&statement.consequent);
            if let Some(alternate) = &statement.alternate {
                visitor.visit_statement(alternate);
            }
        });
    }

    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        self.visit_expression(&expression.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.consequent);
            visitor.visit_expression(&expression.alternate);
        });
    }

    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.right)
        });
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let mut shadowed = HashSet::new();
        if let Some(identifier) = &function.id {
            shadowed.insert(identifier.name.to_string());
        }
        for parameter in &function.params.items {
            collect_binding_names(&parameter.pattern, &mut shadowed);
        }
        if let Some(rest) = &function.params.rest {
            collect_binding_names(&rest.rest.argument, &mut shadowed);
        }
        if let Some(body) = &function.body {
            collect_function_scope_declarations(&body.statements, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_function(visitor, function, flags)
        });
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let mut shadowed = HashSet::new();
        for parameter in &arrow.params.items {
            collect_binding_names(&parameter.pattern, &mut shadowed);
        }
        if let Some(rest) = &arrow.params.rest {
            collect_binding_names(&rest.rest.argument, &mut shadowed);
        }
        collect_function_scope_declarations(&arrow.body.statements, &mut shadowed);
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_arrow_function_expression(visitor, arrow)
        });
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        let mut shadowed = HashSet::new();
        collect_direct_lexical_declarations(&block.body, &mut shadowed);
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_block_statement(visitor, block)
        });
    }

    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        let mut shadowed = HashSet::new();
        if let Some(parameter) = &clause.param {
            collect_binding_names(&parameter.pattern, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| walk::walk_catch_clause(visitor, clause));
    }

    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        let mut shadowed = HashSet::new();
        if let Some(identifier) = &class.id {
            shadowed.insert(identifier.name.to_string());
        }
        self.with_shadow_scope(shadowed, |visitor| walk::walk_class(visitor, class));
    }

    fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
        self.visit_expression(&switch.discriminant);
        let mut shadowed = HashSet::new();
        for case in &switch.cases {
            collect_direct_lexical_declarations(&case.consequent, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| {
            visitor.visit_switch_cases(&switch.cases)
        });
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        let mut shadowed = HashSet::new();
        if let Some(ForStatementInit::VariableDeclaration(declaration)) = &statement.init {
            collect_lexical_variable_names(declaration, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_for_statement(visitor, statement)
        });
    }

    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        let mut shadowed = HashSet::new();
        if let ForStatementLeft::VariableDeclaration(declaration) = &statement.left {
            collect_lexical_variable_names(declaration, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_for_in_statement(visitor, statement)
        });
    }

    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        let mut shadowed = HashSet::new();
        if let ForStatementLeft::VariableDeclaration(declaration) = &statement.left {
            collect_lexical_variable_names(declaration, &mut shadowed);
        }
        self.with_shadow_scope(shadowed, |visitor| {
            walk::walk_for_of_statement(visitor, statement)
        });
    }
}

#[cfg(test)]
mod tests;
