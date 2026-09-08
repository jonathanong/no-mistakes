use super::bindings::callee_name;
use super::{EmbeddedSqlCall, EmbeddedSqlKind};
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, BlockStatement, CallExpression, FormalParameters, Function,
    FunctionBody, Program,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

mod resolve;

#[derive(Clone)]
struct BindingState {
    sql: Option<String>,
    kind: EmbeddedSqlKind,
    line: u32,
}

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
        control_depth: 0,
        functions: resolve::LocalFunctions::collect(program),
    };
    visitor.visit_program(program);
    visitor.calls
}

struct ScopeVisitor<'a> {
    source: &'a str,
    bindings: &'a HashSet<String>,
    scopes: Vec<HashMap<String, BindingState>>,
    calls: Vec<EmbeddedSqlCall>,
    control_depth: usize,
    functions: resolve::LocalFunctions,
}

impl ScopeVisitor<'_> {
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&mut self) -> Option<&mut HashMap<String, BindingState>> {
        self.scopes.last_mut()
    }

    fn lookup(&self, name: &str) -> Option<BindingState> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    /// Whether `name` is bound at a scope more deeply nested than the
    /// top-level program scope — a real lexical shadow of a same-named
    /// top-level helper. A match found only in the outermost scope is the
    /// helper's own top-level declaration (JS/TS forbids redeclaring a name
    /// twice in one scope), not a shadow, and must not block resolving it.
    pub(super) fn shadowed_locally(&self, name: &str) -> bool {
        self.scopes
            .get(1..)
            .is_some_and(|nested| nested.iter().any(|scope| scope.contains_key(name)))
    }

    fn bind_param(&mut self, pattern: &BindingPattern<'_>) {
        let mut names = Vec::new();
        resolve::for_each_bound_name(pattern, &mut |name| names.push(name.to_string()));
        for name in names {
            if let Some(scope) = self.current_scope() {
                scope.insert(
                    name,
                    BindingState {
                        sql: None,
                        kind: EmbeddedSqlKind::Dynamic,
                        line: 0,
                    },
                );
            }
        }
    }

    fn mark_dynamic(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = EmbeddedSqlKind::Dynamic;
                binding.sql = None;
                return;
            }
        }
    }

    fn with_control_flow(&mut self, walk: impl FnOnce(&mut Self)) {
        self.control_depth += 1;
        walk(self);
        self.control_depth = self.control_depth.saturating_sub(1);
    }
}

impl<'a> Visit<'a> for ScopeVisitor<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.push_scope();
        resolve::record_statements(&program.body, self);
        walk::walk_program(self, program);
        self.pop_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_scope();
        resolve::record_statements(&block.body, self);
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        record_params(&function.params, self);
        self.with_control_flow(|visitor| walk::walk_function(visitor, function, flags));
        self.pop_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        resolve::record_statements(&body.statements, self);
        self.with_control_flow(|visitor| walk::walk_function_body(visitor, body));
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_scope();
        record_params(&arrow.params, self);
        self.with_control_flow(|visitor| walk::walk_arrow_function_expression(visitor, arrow));
        self.pop_scope();
    }

    fn visit_assignment_expression(&mut self, assign: &oxc_ast::ast::AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            self.mark_dynamic(ident.name.as_str());
        }
        walk::walk_assignment_expression(self, assign);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        resolve::apply_append(self, call);
        if let Some(callee) = callee_name(call, self.bindings) {
            self.calls.push(resolve::executor_call(self, call, callee));
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_if_statement(visitor, statement));
    }

    fn visit_for_statement(&mut self, statement: &oxc_ast::ast::ForStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_for_statement(visitor, statement));
    }

    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_for_in_statement(visitor, statement));
    }

    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_for_of_statement(visitor, statement));
    }

    fn visit_while_statement(&mut self, statement: &oxc_ast::ast::WhileStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_while_statement(visitor, statement));
    }

    fn visit_do_while_statement(&mut self, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_do_while_statement(visitor, statement));
    }

    fn visit_switch_statement(&mut self, statement: &oxc_ast::ast::SwitchStatement<'a>) {
        self.visit_expression(&statement.discriminant);
        self.with_control_flow(|visitor| visitor.visit_switch_cases(&statement.cases));
    }

    fn visit_conditional_expression(&mut self, expr: &oxc_ast::ast::ConditionalExpression<'a>) {
        self.visit_expression(&expr.test);
        self.with_control_flow(|visitor| {
            visitor.visit_expression(&expr.consequent);
            visitor.visit_expression(&expr.alternate);
        });
    }

    fn visit_logical_expression(&mut self, expr: &oxc_ast::ast::LogicalExpression<'a>) {
        self.visit_expression(&expr.left);
        self.with_control_flow(|visitor| visitor.visit_expression(&expr.right));
    }
}

fn record_params(params: &FormalParameters<'_>, visitor: &mut ScopeVisitor<'_>) {
    for param in &params.items {
        visitor.bind_param(&param.pattern);
    }
}
