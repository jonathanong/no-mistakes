use super::expr::{arg_expr, call_member, pattern_names, unwrap_expr};
use super::extract::ExtractCtx;
use crate::queue::extract_model::ProducerBinding;
use oxc_ast::ast::{
    ArrowFunctionExpression, BlockStatement, CallExpression, Expression, FormalParameters,
    Function, IdentifierReference, Statement, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};

#[derive(Clone, Copy)]
pub(super) enum ActionMode {
    Guarded,
    Listing,
}

pub(super) fn find_in_stmts(
    ctx: &mut ExtractCtx<'_, '_>,
    stmts: &[Statement<'_>],
    job_names: &[String],
    arg_names: &[String],
    mode: ActionMode,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder::new(ctx, job_names, arg_names, mode);
    for stmt in stmts {
        finder.visit_statement(stmt);
        if finder.found.is_some() {
            break;
        }
    }
    finder.found
}

pub(super) fn find_in_expr(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    job_names: &[String],
    arg_names: &[String],
    mode: ActionMode,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder::new(ctx, job_names, arg_names, mode);
    finder.visit_expression(expr);
    finder.found
}

struct ActionFinder<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
    job_names: Vec<String>,
    arg_names: Vec<String>,
    mode: ActionMode,
    found: Option<Option<ProducerBinding>>,
}

impl<'a, 'b, 'c> ActionFinder<'a, 'b, 'c> {
    fn new(
        ctx: &'a mut ExtractCtx<'b, 'c>,
        job_names: &[String],
        arg_names: &[String],
        mode: ActionMode,
    ) -> Self {
        Self {
            ctx,
            job_names: job_names.to_vec(),
            arg_names: arg_names.to_vec(),
            mode,
            found: None,
        }
    }

    fn allow_remove(&self) -> bool {
        matches!(self.mode, ActionMode::Guarded)
    }

    fn require_arg_ref(&self) -> bool {
        matches!(self.mode, ActionMode::Listing)
    }
}

impl<'a> Visit<'a> for ActionFinder<'_, '_, '_> {
    fn visit_function(&mut self, _func: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        let saved_jobs = self.job_names.clone();
        let saved_args = self.arg_names.clone();
        walk::walk_block_statement(self, block);
        self.job_names = saved_jobs;
        self.arg_names = saved_args;
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        let names = pattern_names(&decl.id);
        let Some(init) = &decl.init else {
            return;
        };
        self.visit_expression(init);
        self.job_names.retain(|n| !names.contains(n));
        self.arg_names.retain(|n| !names.contains(n));
        if let Expression::Identifier(src) = unwrap_expr(init) {
            if self.job_names.iter().any(|name| name == src.name.as_str()) {
                self.job_names.extend(names.iter().cloned());
                self.arg_names.extend(names);
                return;
            }
        }
        if self.require_arg_ref() && expr_mentions_any(init, &self.arg_names) {
            self.arg_names.extend(names);
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.found.is_some() {
            return;
        }
        match call.callee.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) => {
                self.visit_arrow_function_body(&arrow.body);
                return;
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    self.visit_function_body(body);
                }
                return;
            }
            _ => {}
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry" || (self.allow_remove() && method == "remove");
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.job_names.iter().any(|name| name == id.name.as_str()) {
                        self.found = Some(None);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                let feeds = !self.require_arg_ref()
                    || call
                        .arguments
                        .iter()
                        .filter_map(arg_expr)
                        .any(|expr| expr_mentions_any(expr, &self.arg_names));
                if feeds {
                    if let Some(binding) = self.ctx.queue_receiver(object, call.span.start) {
                        self.found = Some(Some(binding));
                        return;
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn expr_mentions_any(expr: &Expression<'_>, names: &[String]) -> bool {
    struct RefFinder {
        names: Vec<String>,
        found: bool,
    }
    impl RefFinder {
        fn without_params(&mut self, params: &FormalParameters<'_>) -> Vec<String> {
            let saved = self.names.clone();
            let hidden: Vec<_> = params
                .items
                .iter()
                .flat_map(|p| pattern_names(&p.pattern))
                .collect();
            self.names.retain(|name| !hidden.contains(name));
            saved
        }
    }
    impl<'a> Visit<'a> for RefFinder {
        fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
            if !self.found && self.names.iter().any(|name| name == id.name.as_str()) {
                self.found = true;
            }
        }

        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
            let saved = self.without_params(&arrow.params);
            walk::walk_arrow_function_expression(self, arrow);
            self.names = saved;
        }

        fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
            let saved = self.without_params(&func.params);
            walk::walk_function(self, func, flags);
            self.names = saved;
        }
    }
    let mut finder = RefFinder {
        names: names.to_vec(),
        found: false,
    };
    finder.visit_expression(expr);
    finder.found
}
