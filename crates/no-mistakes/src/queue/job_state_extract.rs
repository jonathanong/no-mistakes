use super::stmts::analyze_statements;
use super::{JobStatePattern, JobStateSite};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, Function, Program, Statement};
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
    let mut ns_imports = Vec::new();
    {
        let mut ctx = ExtractCtx {
            visitor,
            sites: &mut sites,
            seen: &mut seen,
            ns_imports: &mut ns_imports,
        };
        analyze_block(&mut ctx, &program.body);
        FnCollector { ctx: &mut ctx }.visit_program(program);
    }
    for (binding, specifier, member) in ns_imports {
        visitor
            .facts
            .string_imports
            .entry(binding)
            .or_insert((specifier, member));
    }
    visitor.facts.job_state_sites = sites;
}

pub(super) struct ExtractCtx<'a, 'b> {
    pub visitor: &'a QueueVisitor<'b>,
    pub sites: &'a mut Vec<JobStateSite>,
    pub seen: &'a mut HashSet<(usize, JobStatePattern)>,
    pub ns_imports: &'a mut Vec<(String, String, String)>,
}

impl ExtractCtx<'_, '_> {
    pub(super) fn queue_receiver(
        &mut self,
        object: &Expression<'_>,
        at: u32,
    ) -> Option<ProducerBinding> {
        match unwrap_ts_wrappers(object) {
            Expression::Identifier(id) => Some(self.visitor.producer_binding(id.name.as_str(), at)),
            other => {
                if let Expression::StaticMemberExpression(member) = other {
                    if let Expression::Identifier(root) = unwrap_ts_wrappers(&member.object) {
                        let ns = root.name.as_str();
                        if let Some(specifier) = self.visitor.namespace_imports.get(ns) {
                            if matches!(
                                self.visitor.producer_binding(ns, at),
                                ProducerBinding::Local(_)
                            ) {
                                return None;
                            }
                            let prop = member.property.name.as_str();
                            let binding = format!("{ns}.{prop}");
                            self.ns_imports.push((
                                binding.clone(),
                                specifier.clone(),
                                prop.to_string(),
                            ));
                            return Some(ProducerBinding::Named(binding));
                        }
                    }
                }
                self.visitor
                    .imported_member_binding(other)
                    .map(ProducerBinding::Named)
            }
        }
    }

    pub(super) fn record(
        &mut self,
        start: u32,
        binding: ProducerBinding,
        action_binding: Option<ProducerBinding>,
        pattern: JobStatePattern,
    ) {
        let line = line_number(self.visitor.source, start);
        if self.seen.insert((line, pattern)) {
            self.sites.push(JobStateSite {
                line,
                binding,
                action_binding,
                queue_file: None,
                queue_name: None,
                unresolved_receiver: false,
                pattern,
            });
        }
    }
}

struct FnCollector<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
}

impl<'a> Visit<'a> for FnCollector<'_, '_, '_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(body) = &func.body {
            analyze_block(self.ctx, &body.statements);
        }
        walk::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(statements) = crate::ast::arrow_function_body_statements(&arrow.body) {
            analyze_block(self.ctx, statements);
        }
        walk::walk_arrow_function_expression(self, arrow);
    }
}

#[derive(Clone)]
pub(super) struct TrackedJob {
    pub id: usize,
    pub binding: ProducerBinding,
}

#[derive(Clone)]
pub(super) struct StateMatch {
    pub job_id: Option<usize>,
    pub binding: ProducerBinding,
}

#[derive(Default)]
pub(super) struct FnEnv {
    pub next_job_id: usize,
    pub job_vars: HashMap<String, TrackedJob>,
    pub state_vars: HashMap<String, StateMatch>,
    pub listing_vars: HashMap<String, ProducerBinding>,
}

impl FnEnv {
    pub(super) fn alloc_job(&mut self, name: String, binding: ProducerBinding) {
        let id = self.next_job_id;
        self.next_job_id += 1;
        self.job_vars.insert(name, TrackedJob { id, binding });
    }

    pub(super) fn job_names_for(&self, job_id: Option<usize>) -> Vec<String> {
        let Some(target) = job_id else {
            return Vec::new();
        };
        self.job_vars
            .iter()
            .filter(|(_, tracked)| tracked.id == target)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

fn analyze_block(ctx: &mut ExtractCtx<'_, '_>, stmts: &[Statement<'_>]) {
    let mut env = FnEnv::default();
    analyze_statements(ctx, stmts, &mut env);
}
