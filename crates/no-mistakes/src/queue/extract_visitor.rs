use crate::queue::extract_helpers::{binding_name, static_string, JobFrame};
use crate::queue::extract_model::FileFacts;
use crate::queue::extract_record::{record_enqueue, record_flow};
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
}

impl<'a> Visit<'a> for QueueVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.register_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        let Some(name) = binding_name(&decl.id) else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        if let Some(init) = &decl.init {
            if let Some(value) = static_string(init) {
                self.const_strings.insert(name.clone(), value);
            }
        }
        if let Some(Expression::NewExpression(new_expr)) = &decl.init {
            if self.is_queue_constructor(&new_expr.callee) {
                if let Some(queue_name) = new_expr
                    .arguments
                    .first()
                    .and_then(|arg| self.literal_arg(arg))
                {
                    self.facts
                        .queue_bindings
                        .insert(name.clone(), queue_name.clone());
                    self.facts.queue_exports.insert(name.clone(), queue_name);
                }
            } else if self.is_flow_constructor(&new_expr.callee) {
                self.flow_bindings.insert(name.clone());
            }
        }
        if let Some(Expression::CallExpression(call_expr)) = &decl.init {
            super::extract_factory::note_queue_binding(self, &name, call_expr);
        }
        if let Some(Expression::Identifier(id)) = &decl.init {
            super::extract_factory::note_alias(self, &name, id.name.as_str());
        }
        walk::walk_variable_declarator(self, decl);
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
}
