use super::ast::is_cache_directive;
use super::bindings::TopLevelBindings;
use super::patterns::{banned_segment_config, fetch_cache_findings, single_binding_name};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{
    Argument, AssignmentExpression, CallExpression, Declaration, Expression, FunctionBody,
};
use std::collections::{HashMap, HashSet};

pub(crate) struct NextjsCachingVisitor<'a> {
    pub(super) source: &'a str,
    pub(crate) findings: Vec<NextjsCachingFinding>,
    unstable_cache_bindings: HashSet<String>,
    next_cache_namespaces: HashSet<String>,
    next_config_bindings: HashMap<String, Vec<(u32, String)>>,
    segment_config_bindings: HashMap<String, String>,
    local_fetch: bool,
    segment_config: bool,
    next_config: bool,
}

impl<'a> NextjsCachingVisitor<'a> {
    pub(super) fn new(
        source: &'a str,
        findings: Vec<NextjsCachingFinding>,
        bindings: TopLevelBindings,
        segment_config: bool,
        next_config: bool,
    ) -> Self {
        Self {
            source,
            findings,
            unstable_cache_bindings: HashSet::new(),
            next_cache_namespaces: HashSet::new(),
            next_config_bindings: bindings.next_config,
            segment_config_bindings: bindings.segment_config,
            local_fetch: bindings.local_fetch,
            segment_config,
            next_config,
        }
    }

    fn push(&mut self, byte_offset: u32, message: impl Into<String>) {
        self.findings.push(NextjsCachingFinding {
            line: byte_offset_to_line(self.source, byte_offset as usize) as usize,
            message: message.into(),
        });
    }

    pub(crate) fn check_fetch_call(&mut self, call: &CallExpression<'a>) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        if callee.name.as_str() != "fetch" || self.local_fetch {
            return;
        }
        let Some(Argument::ObjectExpression(options)) = call.arguments.get(1) else {
            return;
        };
        for finding in fetch_cache_findings(options) {
            self.push(call.span.start, finding);
        }
    }

    pub(crate) fn check_call(&mut self, call: &CallExpression<'a>) {
        match &call.callee {
            Expression::Identifier(callee)
                if self.unstable_cache_bindings.contains(callee.name.as_str()) =>
            {
                self.push(call.span.start, unstable_cache_message());
            }
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "unstable_cache"
                    && self.is_next_cache_namespace(&member.object) =>
            {
                self.push(call.span.start, unstable_cache_message());
            }
            _ => {}
        }
    }

    pub(crate) fn check_import(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let Some(effects) = super::cache_imports::effects(import) else {
            return;
        };
        self.unstable_cache_bindings
            .extend(effects.unstable_cache_bindings);
        self.next_cache_namespaces.extend(effects.namespaces);
        for (start, message) in effects.findings {
            self.push(start, message);
        }
    }

    pub(crate) fn check_export(&mut self, export: &oxc_ast::ast::ExportDeclaration<'a>) {
        if !self.segment_config {
            return;
        }
        let Declaration::VariableDeclaration(var_decl) = &export.declaration else {
            return;
        };
        for decl in &var_decl.declarations {
            let Some(name) = single_binding_name(&decl.id) else {
                continue;
            };
            let Some(init) = decl.init.as_ref() else {
                continue;
            };
            if banned_segment_config(name.as_str(), init) {
                self.push(decl.span.start, segment_config_message(&name));
            }
        }
    }

    pub(crate) fn check_export_specifiers(
        &mut self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        for specifier in &export.specifiers {
            if let Some(message) = self
                .segment_config_bindings
                .get(specifier.local.name().as_str())
            {
                self.push(specifier.span.start, message.clone());
            }
        }
    }

    pub(crate) fn check_default_export(
        &mut self,
        export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if !self.next_config {
            return;
        }
        self.push_next_config_findings(super::config::default_export_findings(
            &export.declaration,
            &self.next_config_bindings,
        ));
    }

    pub(crate) fn check_assignment(&mut self, assignment: &AssignmentExpression<'a>) {
        if !self.next_config {
            return;
        }
        self.push_next_config_findings(super::config::assignment_findings(
            assignment,
            &self.next_config_bindings,
        ));
    }

    fn push_next_config_findings(&mut self, findings: Vec<(u32, String)>) {
        for (start, message) in findings {
            self.push(start, message);
        }
    }

    pub(crate) fn check_function_body_directives(&mut self, body: &FunctionBody<'a>) {
        for directive in &body.directives {
            if is_cache_directive(directive.directive.as_str()) {
                self.push(
                    directive.span.start,
                    "Next.js cache directives are disabled; remove this directive",
                );
            }
        }
    }

    fn is_next_cache_namespace(&self, expr: &Expression<'a>) -> bool {
        matches!(
            expr,
            Expression::Identifier(id) if self.next_cache_namespaces.contains(id.name.as_str())
        )
    }
}

fn unstable_cache_message() -> &'static str {
    "Next.js unstable_cache is disabled; compute this value per request"
}

fn segment_config_message(name: &str) -> String {
    format!("Next.js `{name}` cache segment config is disabled; remove static caching")
}
