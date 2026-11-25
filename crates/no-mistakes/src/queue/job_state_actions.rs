use super::expr::{call_member, pattern_names, unwrap_expr};
use super::extract::ExtractCtx;
use crate::queue::extract_model::ProducerBinding;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, CallExpression, Expression, Function, IdentifierReference,
    Statement, VariableDeclarator,
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
    mode: ActionMode,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder::new(ctx, job_names, mode);
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
    mode: ActionMode,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder::new(ctx, job_names, mode);
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
    fn new(ctx: &'a mut ExtractCtx<'b, 'c>, job_names: &[String], mode: ActionMode) -> Self {
        Self {
            ctx,
            job_names: job_names.to_vec(),
            arg_names: job_names.to_vec(),
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

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if self.found.is_some() {
            return;
        }
        let Some(init) = &decl.init else {
            return;
        };
        self.visit_expression(init);
        if self.found.is_some() {
            return;
        }
        let names = pattern_names(&decl.id);
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
                        .any(|arg| arg_mentions_any(arg, &self.arg_names));
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

fn arg_mentions_any(arg: &Argument<'_>, names: &[String]) -> bool {
    match arg {
        Argument::SpreadElement(spread) => expr_mentions_any(&spread.argument, names),
        other => other
            .as_expression()
            .is_some_and(|expr| expr_mentions_any(expr, names)),
    }
}

fn expr_mentions_any(expr: &Expression<'_>, names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    struct RefFinder<'a> {
        names: &'a [String],
        found: bool,
    }
    impl<'a> Visit<'a> for RefFinder<'_> {
        fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
            if !self.found && self.names.iter().any(|name| name == id.name.as_str()) {
                self.found = true;
            }
        }
    }
    let mut finder = RefFinder {
        names,
        found: false,
    };
    finder.visit_expression(expr);
    finder.found
}
