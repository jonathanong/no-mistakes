use super::{callee_name, first_call_argument, resolve_call_sql, sql_text, EmbeddedSqlCall};
use oxc_ast::ast::{
    BindingPattern, BlockStatement, CallExpression, Declaration, FormalParameters, Function,
    FunctionBody, Program, Statement, VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

pub(super) fn collect_calls(
    program: &Program<'_>,
    source: &str,
    bindings: &HashSet<String>,
) -> Vec<EmbeddedSqlCall> {
    let mut visitor = ScopeVisitor {
        source,
        bindings,
        scopes: Vec::new(),
        calls: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.calls
}

struct ScopeVisitor<'a> {
    source: &'a str,
    bindings: &'a HashSet<String>,
    scopes: Vec<HashMap<String, Option<String>>>,
    calls: Vec<EmbeddedSqlCall>,
}

impl ScopeVisitor<'_> {
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&mut self) -> Option<&mut HashMap<String, Option<String>>> {
        self.scopes.last_mut()
    }

    fn lookup(&self, name: &str) -> Option<String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
            .flatten()
    }

    fn bind_param(&mut self, pattern: &BindingPattern<'_>) {
        if let BindingPattern::BindingIdentifier(ident) = pattern {
            if let Some(scope) = self.current_scope() {
                scope.insert(ident.name.to_string(), None);
            }
        }
    }
}

impl<'a> Visit<'a> for ScopeVisitor<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.push_scope();
        record_statements(&program.body, self.current_scope());
        walk::walk_program(self, program);
        self.pop_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_scope();
        record_statements(&block.body, self.current_scope());
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        record_params(&function.params, self);
        walk::walk_function(self, function, flags);
        self.pop_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        record_statements(&body.statements, self.current_scope());
        walk::walk_function_body(self, body);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_scope();
        record_params(&arrow.params, self);
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_scope();
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(callee) = callee_name(call, self.bindings) {
            let sql = first_call_argument(call).and_then(|argument| {
                let identifier_bindings = flatten_scopes(&self.scopes);
                resolve_call_sql(argument, &identifier_bindings)
                    .or_else(|| self.lookup_ident(argument))
            });
            self.calls.push(EmbeddedSqlCall {
                line: crate::codebase::ts_source::byte_offset_to_line(
                    self.source,
                    call.span.start as usize,
                ),
                callee,
                sql_text: sql,
            });
        }
        walk::walk_call_expression(self, call);
    }
}

impl ScopeVisitor<'_> {
    fn lookup_ident(&self, argument: &oxc_ast::ast::Expression<'_>) -> Option<String> {
        match crate::codebase::ts_source::unwrap_ts_wrappers(argument) {
            oxc_ast::ast::Expression::Identifier(ident) => self.lookup(ident.name.as_str()),
            _ => None,
        }
    }
}

fn flatten_scopes(scopes: &[HashMap<String, Option<String>>]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for scope in scopes {
        for (name, value) in scope {
            if let Some(text) = value {
                out.insert(name.clone(), text.clone());
            } else {
                out.remove(name);
            }
        }
    }
    out
}

fn record_params(params: &FormalParameters<'_>, visitor: &mut ScopeVisitor<'_>) {
    for param in &params.items {
        visitor.bind_param(&param.pattern);
    }
}

fn record_statements(
    statements: &[Statement<'_>],
    scope: Option<&mut HashMap<String, Option<String>>>,
) {
    let Some(scope) = scope else {
        return;
    };
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                record_variable_declaration(declaration, scope);
            }
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(declaration) = &export.declaration {
                    record_variable_declaration(declaration, scope);
                }
            }
            _ => {}
        }
    }
}

fn record_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    scope: &mut HashMap<String, Option<String>>,
) {
    for declarator in &declaration.declarations {
        record_declarator(declarator, scope);
    }
}

fn record_declarator(
    declarator: &VariableDeclarator<'_>,
    scope: &mut HashMap<String, Option<String>>,
) {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return;
    };
    scope.insert(
        ident.name.to_string(),
        declarator.init.as_ref().and_then(sql_text),
    );
}
