use super::bindings::{callee_name, sql_statement_type_bindings};
use super::{EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlKind};
use oxc_ast::ast::{
    AssignmentTarget, BlockStatement, CallExpression, Function, FunctionBody, FunctionType,
    Program, ReturnStatement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

mod resolve;
mod scope;

#[derive(Clone)]
struct BindingState {
    sql: Option<String>,
    kind: EmbeddedSqlKind,
    line: u32,
    sql_builder: bool,
}

pub(super) fn collect_calls(
    program: &Program<'_>,
    source: &str,
    bindings: &HashSet<String>,
) -> (Vec<EmbeddedSqlCall>, Vec<EmbeddedSqlFragment>) {
    let mut visitor = ScopeVisitor {
        source,
        bindings,
        scopes: Vec::new(),
        calls: Vec::new(),
        fragments: Vec::new(),
        suppress_nested_builder_fragments: 0,
        control_depth: 0,
        loop_depth: 0,
        function_scopes: Vec::new(),
        functions: resolve::LocalFunctions::collect(program),
        sql_statement_types: sql_statement_type_bindings(program),
    };
    visitor.visit_program(program);
    (visitor.calls, visitor.fragments)
}

struct ScopeVisitor<'a> {
    source: &'a str,
    bindings: &'a HashSet<String>,
    scopes: Vec<HashMap<String, BindingState>>,
    calls: Vec<EmbeddedSqlCall>,
    fragments: Vec<EmbeddedSqlFragment>,
    suppress_nested_builder_fragments: usize,
    control_depth: usize,
    loop_depth: usize,
    function_scopes: Vec<usize>,
    functions: resolve::LocalFunctions,
    sql_statement_types: HashSet<String>,
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

    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        self.push_scope();
        if let Some(param) = &clause.param {
            self.bind_param(&param.pattern, false);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_scope();
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        self.record_params(&function.params);
        // A named function expression's own name is visible only inside its
        // own body (unlike a declaration's, hoisted into the enclosing
        // scope by `record_function_declaration`), so it belongs in the
        // scope this call just pushed rather than in any outer one. Without
        // it, `shadowed_locally` can never see that the name is rebound here
        // at all, and a same-spelled reference inside the body — e.g. using
        // the expression's own name as a template tag — reads back as the
        // untouched top-level/global binding instead of this local rebind.
        if function.r#type == FunctionType::FunctionExpression {
            if let Some(id) = &function.id {
                self.bind_self_name(id.name.as_str());
            }
        }
        self.enter_function();
        walk::walk_function(self, function, flags);
        self.leave_function();
        self.pop_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        resolve::record_statements(&body.statements, self);
        walk::walk_function_body(self, body);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_scope();
        self.record_params(&arrow.params);
        self.enter_function();
        walk::walk_arrow_function_expression(self, arrow);
        self.leave_function();
        self.pop_scope();
    }

    fn visit_assignment_expression(&mut self, assign: &oxc_ast::ast::AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            self.mark_dynamic(ident.name.as_str());
        }
        walk::walk_assignment_expression(self, assign);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.suppress_nested_builder_fragments == 0 {
            if let Some(argument) = call.arguments.first() {
                if let Some(sql_text) = resolve::appended_builder_fragment(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        Some(sql_text),
                    );
                } else if resolve::is_builder_append(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        None,
                    );
                }
            }
        }
        // Capture a fragment before applying the mutation: an append inside
        // a loop makes the receiver dynamic for subsequent observations, but
        // the append call itself still has trusted SQL-builder provenance.
        resolve::apply_append(self, call);
        if let Some(callee) = callee_name(call, self.bindings) {
            self.calls.push(resolve::executor_call(self, call, callee));
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if let Some(argument) = &statement.argument {
            if let Some(sql_text) = resolve::builder_fragment(argument, self) {
                self.push_fragment(
                    crate::codebase::ts_source::byte_offset_to_line(
                        self.source,
                        argument.span().start as usize,
                    ),
                    Some(sql_text),
                );
                self.suppress_nested_builder_fragments += 1;
                walk::walk_return_statement(self, statement);
                self.suppress_nested_builder_fragments -= 1;
                return;
            }
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_for_statement(&mut self, statement: &oxc_ast::ast::ForStatement<'a>) {
        resolve::enter_classic_for(statement, self);
        self.with_loop(|visitor| walk::walk_for_statement(visitor, statement));
        resolve::leave_classic_for(statement, self);
    }

    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_loop(|visitor| walk::walk_for_in_statement(visitor, statement));
        self.pop_scope();
    }

    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_loop(|visitor| walk::walk_for_of_statement(visitor, statement));
        self.pop_scope();
    }

    fn visit_while_statement(&mut self, statement: &oxc_ast::ast::WhileStatement<'a>) {
        self.with_loop(|visitor| walk::walk_while_statement(visitor, statement));
    }

    fn visit_do_while_statement(&mut self, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.with_loop(|visitor| walk::walk_do_while_statement(visitor, statement));
    }

    fn visit_switch_statement(&mut self, statement: &oxc_ast::ast::SwitchStatement<'a>) {
        self.visit_expression(&statement.discriminant);
        // All of a switch's cases share one lexical scope (unlike a
        // `BlockStatement` per case), so a case-local function declaration
        // is recorded here, once, across every case's statements — not
        // per-case — before any case is walked. Without this, a `function
        // build() {}` declared directly in a case body is invisible to
        // `shadowed_locally`, and a same-named top-level helper is wrongly
        // resolved through instead.
        self.push_scope();
        for case in &statement.cases {
            resolve::record_statements(&case.consequent, self);
        }
        self.with_control_flow(|visitor| visitor.visit_switch_cases(&statement.cases));
        self.pop_scope();
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.with_control_flow(|visitor| walk::walk_if_statement(visitor, statement));
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

impl ScopeVisitor<'_> {
    fn push_fragment(&mut self, line: u32, sql_text: Option<String>) {
        self.fragments.push(EmbeddedSqlFragment { line, sql_text });
    }
}
