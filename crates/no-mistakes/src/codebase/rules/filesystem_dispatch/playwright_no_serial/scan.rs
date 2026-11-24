use super::super::test_flake_helpers::{
    ast::{
        array_objects, bool_literal, collect_branch_exprs, find_property, member_chain,
        non_negative_int, project_name, string_literal,
    },
    config_ast::extract_config_objects,
    is_script, make_finding,
};
use super::{CompiledOptions, RULE_ID};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, unwrap_ts_wrappers, SourceStore};
use oxc_ast::ast::{CallExpression, Expression, ObjectExpression};
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::path::Path;

pub(super) fn scan_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !is_script(&rel) {
        return Vec::new();
    }
    let Some(source) = rules::read_language_source(sources, path) else {
        return Vec::new();
    };
    scan_source(&rel, &source, opts)
}

pub(super) fn scan_source(file: &str, source: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let is_config = opts.is_playwright_config(file);
    crate::ast::with_program(Path::new(file), source, |program, source| {
        let mut findings = Vec::new();
        if is_config {
            for obj in extract_config_objects(program) {
                scan_config_object(file, source, obj, opts, &mut findings);
            }
        }
        if !opts.allow.is_allowed(file, None) {
            let mut visitor = TestVisitor {
                file,
                source,
                findings: &mut findings,
            };
            visitor.visit_program(program);
        }
        findings
    })
    .unwrap_or_default()
}

fn scan_config_object(
    file: &str,
    source: &str,
    obj: &ObjectExpression<'_>,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    if !opts.allow.is_allowed(file, None) {
        check_serial_config_props(file, source, obj, None, findings);
    }
    let Some(projects_prop) = find_property(obj, "projects") else {
        return;
    };
    for project_obj in array_objects(&projects_prop.value) {
        let name = project_name(project_obj);
        if opts.allow.is_allowed(file, name) {
            continue;
        }
        check_serial_config_props(file, source, project_obj, name, findings);
    }
}

fn check_serial_config_props(
    file: &str,
    source: &str,
    obj: &ObjectExpression<'_>,
    project: Option<&str>,
    findings: &mut Vec<RuleFinding>,
) {
    let scope = match project {
        Some(name) => format!("Playwright project `{name}`"),
        None => "Playwright config".to_string(),
    };
    if let Some(prop) = find_property(obj, "workers") {
        let mut branches = Vec::new();
        collect_branch_exprs(&prop.value, &mut branches);
        if let Some((_, span)) = branches
            .into_iter()
            .filter_map(non_negative_int)
            .find(|(value, _)| *value == 1)
        {
            findings.push(make_finding(
                RULE_ID,
                file,
                source,
                span,
                format!(
                    "{scope} sets `workers: 1`, serializing test execution and hiding concurrency flakes. Remove `workers: 1` and isolate shared test state, or add an allow entry with a reason."
                ),
                Some("workers".to_string()),
            ));
        }
    }
    if let Some(prop) = find_property(obj, "fullyParallel") {
        let mut branches = Vec::new();
        collect_branch_exprs(&prop.value, &mut branches);
        if let Some((_, span)) = branches
            .into_iter()
            .filter_map(bool_literal)
            .find(|(value, _)| !*value)
        {
            findings.push(make_finding(
                RULE_ID,
                file,
                source,
                span,
                format!(
                    "{scope} sets `fullyParallel: false`, serializing tests within each file and hiding ordering dependencies. Enable parallel execution and isolate test state, or add an allow entry with a reason."
                ),
                Some("fullyParallel".to_string()),
            ));
        }
    }
}

struct TestVisitor<'a> {
    file: &'a str,
    source: &'a str,
    findings: &'a mut Vec<RuleFinding>,
}

impl<'a> Visit<'a> for TestVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.check_call(call);
        walk::walk_call_expression(self, call);
    }
}

impl TestVisitor<'_> {
    fn check_call(&mut self, call: &CallExpression<'_>) {
        if call.optional {
            return;
        }
        let Some(chain) = member_chain(&call.callee) else {
            return;
        };
        if chain.len() >= 3 && chain[0] == "test" && chain[1] == "describe" && chain[2] == "serial"
        {
            self.push(
                call.span,
                "Playwright `test.describe.serial` forces sequential execution and hides inter-test state coupling. Make each test independent and use `test.describe`, or add an allow entry with a reason.".to_string(),
                "test.describe.serial",
            );
            return;
        }
        if chain.as_slice() == ["test", "describe", "configure"] {
            self.check_configure(call);
        }
    }

    fn check_configure(&mut self, call: &CallExpression<'_>) {
        let Some(arg) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        let Expression::ObjectExpression(obj) = unwrap_ts_wrappers(arg) else {
            return;
        };
        let Some(mode_prop) = find_property(obj, "mode") else {
            return;
        };
        let mut branches = Vec::new();
        collect_branch_exprs(&mode_prop.value, &mut branches);
        if branches
            .into_iter()
            .any(|expr| string_literal(expr) == Some("serial"))
        {
            self.push(
                mode_prop.span,
                "Playwright `test.describe.configure({ mode: 'serial' })` forces sequential execution and hides inter-test state coupling. Make each test independent instead of serializing the suite, or add an allow entry with a reason.".to_string(),
                "test.describe.configure",
            );
        }
    }

    fn push(&mut self, span: Span, message: String, target: &str) {
        self.findings.push(make_finding(
            RULE_ID,
            self.file,
            self.source,
            span,
            message,
            Some(target.to_string()),
        ));
    }
}
