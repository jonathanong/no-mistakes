use crate::queue::extract_helpers::JobFrame;
use crate::queue::extract_model::FileFacts;
use crate::queue::extract_record::{record_enqueue, record_flow};
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};
use std::path::Path;

mod bindings;
mod helpers;

pub(super) struct QueueVisitor<'a> {
    pub path: &'a Path,
    pub source: &'a str,
    pub facts: FileFacts,
    pub const_strings: HashMap<String, String>,
    pub queue_classes: HashSet<String>,
    pub worker_classes: HashSet<String>,
    pub flow_classes: HashSet<String>,
    pub flow_bindings: HashSet<String>,
    pub configured_factory_names: &'a [String],
    pub roles: &'a crate::config::v2::schema::QueueFactoryRoles,
    pub queue_name_index: HashMap<String, usize>,
    pub enqueue_factories: HashMap<String, (usize, usize, Option<usize>)>,
    pub worker_factories: HashMap<String, (usize, usize)>,
    pub job_frames: Vec<JobFrame>,
    pub record_waits: bool,
    pub current_case: Option<String>,
    pub namespace_imports: HashMap<String, String>,
    /// `(processor span start, end, worker index)` for factory and `new Worker` processors.
    pub processor_spans: Vec<(u32, u32, usize)>,
    /// Indexes into `processor_spans` already claimed by an outer function.
    pub claimed_processors: Vec<usize>,
    pub worker_stack: Vec<usize>,
    pub dispatch_jobs: HashMap<usize, Vec<String>>,
    pub dispatch_dynamic_workers: HashSet<usize>,
    /// Innermost function spans where `note_job` or `note_dynamic` fired.
    pub dispatch_scopes: HashSet<u32>,
    /// Function spans from the module root down to the function being visited.
    pub function_stack: Vec<u32>,
    /// Functions and arrows that are not nested inside another function.
    pub top_level_callables: u32,
}

impl<'a> Visit<'a> for QueueVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.register_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        self.note_variable(decl);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            let method = member.property.name.as_str();
            if method == "addAndWait" || method == "waitUntilFinished" {
                if self.record_waits {
                    super::extract_factory::note_wait(self, call);
                }
            } else if let Expression::Identifier(object) = &member.object {
                if method == "add" && self.flow_bindings.contains(object.name.as_str()) {
                    let recorded = self.facts.producers.len();
                    record_flow(
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                } else if method == "add" || method == "addBulk" {
                    let recorded = self.facts.producers.len();
                    record_enqueue(
                        object.name.as_str(),
                        method,
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                    super::extract_factory::note_self_dedup(self, call, recorded);
                }
            } else if method == "add" || method == "addBulk" {
                if let Some(binding) = self.imported_member_binding(&member.object) {
                    let recorded = self.facts.producers.len();
                    record_enqueue(
                        &binding,
                        method,
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    self.stamp_worker(recorded);
                    super::extract_factory::note_self_dedup(self, call, recorded);
                }
            }
        }
        super::extract_factory::note_factory_call(self, call);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new_expr: &oxc_ast::ast::NewExpression<'a>) {
        self.note_constructed_worker(new_expr);
        walk::walk_new_expression(self, new_expr);
    }

    fn visit_export_named_declaration(&mut self, decl: &oxc_ast::ast::ExportNamedDeclaration<'a>) {
        super::extract_factory::note_export(self, decl);
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_from_declaration(&mut self, decl: &oxc_ast::ast::ExportFromDeclaration<'a>) {
        super::extract_factory::note_export_from(self, decl);
        walk::walk_export_from_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &oxc_ast::ast::ExportAllDeclaration<'a>) {
        super::extract_factory::note_export_all(self, decl);
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_export_declaration(&mut self, decl: &oxc_ast::ast::ExportDeclaration<'a>) {
        super::extract_factory::note_exported_declaration(self, decl);
        walk::walk_export_declaration(self, decl);
    }

    fn visit_function(
        &mut self,
        func: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        super::extract_factory::visit_function(self, func, flags);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        super::extract_factory::visit_arrow(self, arrow);
    }

    fn visit_switch_statement(&mut self, stmt: &oxc_ast::ast::SwitchStatement<'a>) {
        super::extract_factory::visit_switch(self, stmt);
    }

    fn visit_variable_declaration(&mut self, decl: &oxc_ast::ast::VariableDeclaration<'a>) {
        super::extract_factory::visit_variable_declaration(self, decl);
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        super::extract_factory::visit_block(self, block);
    }

    fn visit_binary_expression(&mut self, expr: &oxc_ast::ast::BinaryExpression<'a>) {
        super::extract_processor::visit_binary(self, expr);
    }

    fn visit_static_member_expression(
        &mut self,
        member: &oxc_ast::ast::StaticMemberExpression<'a>,
    ) {
        super::extract_processor::visit_member(self, member);
    }

    fn visit_computed_member_expression(
        &mut self,
        member: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        super::extract_processor::visit_computed_member(self, member);
    }

    fn visit_identifier_reference(&mut self, id: &oxc_ast::ast::IdentifierReference<'a>) {
        super::extract_processor::visit_identifier(self, id);
    }
    fn visit_function_body(&mut self, body: &oxc_ast::ast::FunctionBody<'a>) {
        super::extract_factory::visit_body(self, body);
    }
    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        super::extract_factory::visit_catch(self, clause);
    }
    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        super::extract_factory::visit_class(self, class);
    }
}
