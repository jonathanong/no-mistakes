use super::{Collector, ScopeKind};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, BlockStatement, CatchClause, Expression,
    ForInStatement, ForOfStatement, ForStatement, Function, FunctionType, Program, Statement,
    VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn stored_expr<'a>(expr: &Expression<'a>) -> Option<&'a Expression<'a>> {
    let peeled = crate::codebase::ts_source::unwrap_ts_wrappers(arena_expr(expr));
    match peeled {
        Expression::ObjectExpression(_) | Expression::Identifier(_) => Some(peeled),
        _ => None,
    }
}

fn arena_expr<'a>(expr: &Expression<'a>) -> &'a Expression<'a> {
    // SAFETY: oxc allocates this node in the program arena. The visitor lends a
    // shorter borrow of that same node, and the binding list ends with the program.
    unsafe { &*(expr as *const Expression<'a>) }
}

pub(super) fn bind_pattern<'a>(
    collector: &mut Collector<'a>,
    pattern: &BindingPattern<'a>,
    scope_start: u32,
    scope_end: u32,
    visible_from: u32,
    resolve_at: u32,
    value: Option<&'a Expression<'a>>,
) {
    if let BindingPattern::BindingIdentifier(id) = pattern {
        collector.bindings.push(super::ObjectBinding {
            name: id.name.as_str(),
            scope_start,
            scope_end,
            visible_from,
            resolve_at,
            value,
        });
        return;
    }
    let mut names = Vec::new();
    collect_names(pattern, &mut names);
    for name in names {
        collector.bindings.push(super::ObjectBinding {
            name,
            scope_start,
            scope_end,
            visible_from,
            resolve_at: visible_from,
            value: None,
        });
    }
}

pub(super) fn collect_names<'a>(pattern: &BindingPattern<'a>, names: &mut Vec<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => names.push(id.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => collect_names(&assignment.left, names),
    }
}

impl<'a> Visit<'a> for Collector<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.push_scope(program.span.start, program.span.end, ScopeKind::Function);
        walk::walk_program(self, program);
        self.scopes.pop();
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        if func.r#type == FunctionType::FunctionDeclaration {
            if let Some(id) = &func.id {
                let start = self.current().start;
                self.mask(id.name.as_str(), start);
            }
        }
        self.push_scope(func.span.start, func.span.end, ScopeKind::Function);
        if func.r#type == FunctionType::FunctionExpression {
            if let Some(id) = &func.id {
                self.mask(id.name.as_str(), func.span.start);
            }
        }
        self.mask_params(&func.params, func.span.start);
        walk::walk_function(self, func, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.push_scope(arrow.span.start, arrow.span.end, ScopeKind::Arrow);
        self.mask_params(&arrow.params, arrow.span.start);
        walk::walk_arrow_function_expression(self, arrow);
        self.scopes.pop();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_scope(block.span.start, block.span.end, ScopeKind::Block);
        walk::walk_block_statement(self, block);
        self.scopes.pop();
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.push_scope(clause.span.start, clause.span.end, ScopeKind::Block);
        if let Some(param) = &clause.param {
            self.mask_pattern(&param.pattern, clause.span.start);
        }
        walk::walk_catch_clause(self, clause);
        self.scopes.pop();
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        self.push_scope(stmt.span.start, stmt.span.end, ScopeKind::Block);
        walk::walk_for_statement(self, stmt);
        self.scopes.pop();
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        self.push_scope(stmt.span.start, stmt.span.end, ScopeKind::Block);
        walk::walk_for_in_statement(self, stmt);
        self.scopes.pop();
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        self.push_scope(stmt.span.start, stmt.span.end, ScopeKind::Block);
        walk::walk_for_of_statement(self, stmt);
        self.scopes.pop();
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        if let Statement::ClassDeclaration(class) = stmt {
            if let Some(id) = &class.id {
                let start = self.current().start;
                self.mask(id.name.as_str(), start);
            }
        }
        walk::walk_statement(self, stmt);
    }

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        for declarator in &decl.declarations {
            self.record_declarator(decl.kind, declarator);
        }
        walk::walk_variable_declaration(self, decl);
    }
}
