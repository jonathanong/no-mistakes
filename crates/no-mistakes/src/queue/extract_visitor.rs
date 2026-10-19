use crate::queue::extract_helpers::{
    binding_name, collect_jobs_from_args, export_name, is_queue_package, processor_specifier,
};
use crate::queue::extract_model::{FileFacts, ImportBinding, WorkerSite};
use crate::queue::extract_record::{record_enqueue, record_flow};
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, ImportDeclarationSpecifier};
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
    pub enqueue_factories: HashMap<String, (usize, usize)>,
    pub worker_factories: HashMap<String, (usize, usize)>,
    pub in_job: u32,
    pub current_case: Option<String>,
    pub namespace_imports: HashMap<String, String>,
}

impl<'a> Visit<'a> for QueueVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let source = import.source.value.as_str().to_string();
        if let Some(specifiers) = &import.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        let imported = export_name(&spec.imported);
                        let local = spec.local.name.as_str().to_string();
                        if is_queue_package(&source)
                            && matches!(imported.as_str(), "Queue" | "TestQueue")
                        {
                            self.queue_classes.insert(local.clone());
                        }
                        if is_queue_package(&source)
                            && matches!(imported.as_str(), "Worker" | "TestWorker")
                        {
                            self.worker_classes.insert(local.clone());
                        }
                        if is_queue_package(&source) && imported == "FlowProducer" {
                            self.flow_classes.insert(local.clone());
                        }
                        super::extract_factory::note_named_import(self, &imported, &local, &source);
                        self.facts.imports.push(ImportBinding {
                            local,
                            imported,
                            source: source.clone(),
                        });
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        self.facts.imports.push(ImportBinding {
                            local: spec.local.name.as_str().to_string(),
                            imported: "default".to_string(),
                            source: source.clone(),
                        });
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        self.namespace_imports
                            .insert(spec.local.name.as_str().to_string(), source.clone());
                    }
                }
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        let Some(name) = binding_name(&decl.id) else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        if let Some(Expression::StringLiteral(value)) = &decl.init {
            self.const_strings
                .insert(name.clone(), value.value.as_str().to_string());
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
            if let Expression::Identifier(object) = &member.object {
                let method = member.property.name.as_str();
                if method == "add" && self.flow_bindings.contains(object.name.as_str()) {
                    record_flow(
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                } else if method == "add" || method == "addBulk" {
                    record_enqueue(
                        object.name.as_str(),
                        method,
                        self.path,
                        self.source,
                        &self.const_strings,
                        &mut self.facts,
                        call,
                    );
                    super::extract_factory::note_self_dedup(self, call);
                } else if method == "addAndWait" || method == "waitUntilFinished" {
                    super::extract_factory::note_wait(self, call);
                }
            }
        }
        super::extract_factory::note_factory_call(self, call);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new_expr: &oxc_ast::ast::NewExpression<'a>) {
        if self.is_worker_constructor(&new_expr.callee) {
            let queue_name = new_expr
                .arguments
                .first()
                .and_then(|arg| self.literal_arg(arg));
            let mut jobs = collect_jobs_from_args(&new_expr.arguments, self.source);
            jobs.sort();
            jobs.dedup();
            let processor_specifier =
                processor_specifier(&new_expr.arguments, self.source, &self.namespace_imports);
            self.facts.workers.push(WorkerSite {
                file: self.path.to_path_buf(),
                line: line_number(self.source, new_expr.span.start),
                queue_name,
                queue_name_import: None,
                wildcard: jobs.is_empty(),
                jobs,
                processor_specifier,
                processor_file: None,
            });
        }
        walk::walk_new_expression(self, new_expr);
    }

    fn visit_export_named_declaration(&mut self, decl: &oxc_ast::ast::ExportNamedDeclaration<'a>) {
        super::extract_factory::note_export(self, decl);
        walk::walk_export_named_declaration(self, decl);
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
