use super::effect_calls::{declarator_function_name, record_effect, EffectNames, EffectSink};
use super::{EffectCallFact, TsFactPlan};
use crate::codebase::ts_http_calls::{export_default_allows_http, record_http_call, HttpCall};
use crate::codebase::ts_source::facts::call_sites::{record_call_site, CallSiteFact};
use crate::codebase::ts_trpc::{finish_trpc_calls, procedure_path_from_call, TrpcCallFact};
use oxc_ast::ast::{
    CallExpression, ExportDefaultDeclaration, Function, NewExpression, Program, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

#[derive(Default)]
pub(super) struct FusedDomainCalls {
    pub http_calls: Vec<HttpCall>,
    pub effect_calls: Vec<EffectCallFact>,
    pub trpc_calls: Vec<TrpcCallFact>,
    pub call_sites: Vec<CallSiteFact>,
}

pub(super) fn collect_fused_domain_calls(
    program: &Program<'_>,
    source: &str,
    plan: TsFactPlan,
    http_prefixes: &[&str],
    effect_names: &EffectNames,
) -> FusedDomainCalls {
    if !plan.http_calls && !plan.effect_calls && !plan.trpc_calls && !plan.call_sites {
        return FusedDomainCalls::default();
    }
    let mut visitor = DomainCallVisitor {
        source,
        http_prefixes,
        http_ok: plan.http_calls,
        collect_http: plan.http_calls,
        collect_effects: plan.effect_calls,
        collect_trpc: plan.trpc_calls,
        collect_call_sites: plan.call_sites,
        effect_names,
        caller_stack: Vec::new(),
        call_site_scope: Vec::new(),
        hits: FusedDomainCalls::default(),
    };
    visitor.visit_program(program);
    finish_trpc_calls(&mut visitor.hits.trpc_calls);
    visitor.hits
}

struct DomainCallVisitor<'a, 'b> {
    source: &'a str,
    http_prefixes: &'b [&'a str],
    http_ok: bool,
    collect_http: bool,
    collect_effects: bool,
    collect_trpc: bool,
    collect_call_sites: bool,
    effect_names: &'b EffectNames,
    caller_stack: Vec<String>,
    call_site_scope: Vec<String>,
    hits: FusedDomainCalls,
}

impl DomainCallVisitor<'_, '_> {
    fn record_effect(&mut self, callee: &oxc_ast::ast::Expression<'_>, byte_offset: u32) {
        if !self.collect_effects {
            return;
        }
        record_effect(
            EffectSink {
                source: self.source,
                names: self.effect_names,
                caller: self.caller_stack.last(),
                hits: &mut self.hits.effect_calls,
            },
            callee,
            byte_offset,
        );
    }

    fn record_call(&mut self, call: &CallExpression<'_>) {
        if self.collect_http && self.http_ok {
            record_http_call(
                call,
                self.source,
                self.http_prefixes,
                &mut self.hits.http_calls,
            );
        }
        self.record_effect(&call.callee, call.span.start);
        if self.collect_trpc {
            if let Some(path) = procedure_path_from_call(call) {
                self.hits.trpc_calls.push(TrpcCallFact { path });
            }
        }
        if self.collect_call_sites {
            record_call_site(
                self.source,
                self.call_site_scope.last(),
                call,
                &mut self.hits.call_sites,
            );
        }
    }
}

impl<'a> Visit<'a> for DomainCallVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        self.record_effect(&new.callee, new.span.start);
        walk::walk_new_expression(self, new);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let name = function.id.as_ref().map(|id| id.name.to_string());
        if let Some(name) = &name {
            self.caller_stack.push(name.clone());
            if self.collect_call_sites {
                self.call_site_scope.push(name.clone());
            }
        }
        walk::walk_function(self, function, flags);
        if name.is_some() {
            self.caller_stack.pop();
            if self.collect_call_sites {
                self.call_site_scope.pop();
            }
        }
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let name = declarator_function_name(declarator);
        if let Some(name) = &name {
            self.caller_stack.push(name.clone());
        }
        walk::walk_variable_declarator(self, declarator);
        if name.is_some() {
            self.caller_stack.pop();
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        let previous = self.http_ok;
        if self.collect_http && !export_default_allows_http(decl) {
            self.http_ok = false;
        }
        walk::walk_export_default_declaration(self, decl);
        self.http_ok = previous;
    }
}
