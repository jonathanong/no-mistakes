use super::{Collector, Value};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, CallExpression, ForOfStatement, ForStatementLeft,
    Function, IfStatement, Statement, StaticBlock, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};

impl<'a> Visit<'a> for Collector<'_, 'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let previous = std::mem::take(&mut self.flow);
        walk::walk_function(self, function, flags);
        self.flow = previous;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let previous = std::mem::take(&mut self.flow);
        walk::walk_arrow_function_expression(self, arrow);
        self.flow = previous;
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        let previous = std::mem::take(&mut self.flow);
        walk::walk_static_block(self, block);
        self.flow = previous;
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        walk::walk_variable_declarator(self, declaration);
        if let Some(symbol) = self.stable_binding(&declaration.id) {
            if self
                .queue
                .scoping
                .is_some_and(|scoping| scoping.symbol_flags(symbol).is_const_variable())
            {
                if let Some(value) = declaration.init.as_ref().and_then(|init| self.value(init)) {
                    self.flow.values.insert(symbol, value);
                }
            }
        }
    }

    fn visit_statements(&mut self, statements: &oxc_allocator::Vec<'a, Statement<'a>>) {
        let guard_count = self.flow.guards.len();
        for statement in statements {
            self.visit_statement(statement);
            if let Statement::IfStatement(branch) = statement {
                if exits(&branch.consequent) != branch.alternate.as_ref().is_some_and(exits) {
                    self.flow.guards.extend(self.guard(&branch.test));
                }
            }
        }
        self.flow.guards.truncate(guard_count);
    }

    fn visit_if_statement(&mut self, branch: &IfStatement<'a>) {
        self.visit_expression(&branch.test);
        let guard_count = self.flow.guards.len();
        self.flow.guards.extend(self.guard(&branch.test));
        self.visit_statement(&branch.consequent);
        if let Some(alternate) = &branch.alternate {
            self.visit_statement(alternate);
        }
        self.flow.guards.truncate(guard_count);
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.visit_for_statement_left(&statement.left);
        self.visit_expression(&statement.right);
        let item = loop_binding(statement).and_then(|pattern| self.stable_binding(pattern));
        let binding = match (item, self.value(&statement.right)) {
            (Some(symbol), Some(Value::Listing(mut origin))) => {
                origin.at = statement.span.start;
                self.flow.loops.push(origin.at);
                self.flow.values.insert(symbol, Value::Item(origin));
                Some(symbol)
            }
            _ => None,
        };
        self.visit_statement(&statement.body);
        if let Some(binding) = binding {
            self.flow.values.remove(&binding);
            self.flow.loops.pop();
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.note_action(call);
        walk::walk_call_expression(self, call);
    }
}

impl Collector<'_, '_> {
    fn stable_binding(&self, pattern: &BindingPattern<'_>) -> Option<SymbolId> {
        let BindingPattern::BindingIdentifier(id) = pattern else {
            return None;
        };
        let symbol = id.symbol_id.get()?;
        let scoping = self.queue.scoping?;
        (!scoping.symbol_is_mutated(symbol) && scoping.symbol_redeclarations(symbol).is_empty())
            .then_some(symbol)
    }
}

fn loop_binding<'node, 'a>(
    statement: &'node ForOfStatement<'a>,
) -> Option<&'node BindingPattern<'a>> {
    let ForStatementLeft::VariableDeclaration(declaration) = &statement.left else {
        return None;
    };
    Some(&declaration.declarations.first()?.id)
}

fn exits(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block.body.last().is_some_and(exits),
        _ => false,
    }
}
