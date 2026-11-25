use super::actions::Action;
use super::stmts::analyze_statements;
use super::{JobStatePattern, JobStateSite};
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{ArrowFunctionExpression, Function, Program, Statement};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};

pub(crate) fn collect<'a>(visitor: &mut QueueVisitor<'a>, program: &Program<'a>) {
    let source = visitor.source;
    if !source.contains("getJob")
        && !source.contains("getFailed")
        && !source.contains("getCompleted")
        && !source.contains("getJobs")
    {
        return;
    }
    let mut sites = Vec::new();
    let mut seen = HashSet::new();
    analyze_block(visitor, &program.body, &mut sites, &mut seen);
    let mut finder = FnCollector {
        visitor,
        sites: &mut sites,
        seen: &mut seen,
    };
    finder.visit_program(program);
    visitor.facts.job_state_sites = sites;
}

struct FnCollector<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    sites: &'b mut Vec<JobStateSite>,
    seen: &'b mut HashSet<(usize, JobStatePattern)>,
}

impl<'a> Visit<'a> for FnCollector<'_, '_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(body) = &func.body {
            analyze_block(self.visitor, &body.statements, self.sites, self.seen);
        }
        walk::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(statements) = crate::ast::arrow_function_body_statements(&arrow.body) {
            analyze_block(self.visitor, statements, self.sites, self.seen);
        }
        walk::walk_arrow_function_expression(self, arrow);
    }
}

#[derive(Default)]
pub(super) struct FnEnv {
    pub job_vars: HashMap<String, ProducerBinding>,
    pub state_vars: HashMap<String, ProducerBinding>,
    pub listing_vars: HashMap<String, ProducerBinding>,
}

fn analyze_block(
    visitor: &QueueVisitor<'_>,
    stmts: &[Statement<'_>],
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    let mut env = FnEnv::default();
    analyze_statements(visitor, stmts, &mut env, sites, seen);
}

pub(super) fn record(
    visitor: &QueueVisitor<'_>,
    start: u32,
    binding: ProducerBinding,
    action: Action,
    pattern: JobStatePattern,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    let line = line_number(visitor.source, start);
    if seen.insert((line, pattern)) {
        let (action_binding, uncertain_item) = action.into_site();
        sites.push(JobStateSite {
            line,
            binding,
            action_binding,
            queue_file: None,
            queue_name: None,
            unresolved_receiver: false,
            uncertain_item,
            pattern,
        });
    }
}
