use super::literals::{
    integer_cardinality, is_named_member, length_expression, member_names, numeric_list,
};
use super::{in_scope, is_script, relative_file, Compiled, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers, SourceStore};
use oxc_ast::ast::{CallExpression, Expression, NewExpression};
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::path::Path;

pub(super) fn scan_file(
    root: &Path,
    path: &Path,
    policy: &Compiled,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_file(root, path);
    if !is_script(&rel) || !in_scope(policy, &rel) {
        return Vec::new();
    }
    let Some(source) = super::super::read_language_source(sources, path) else {
        return Vec::new();
    };
    scan_source(&rel, &source, policy)
}

pub(super) fn scan_source(file: &str, source: &str, policy: &Compiled) -> Vec<RuleFinding> {
    let Some(threshold) = policy.threshold else {
        return Vec::new();
    };
    crate::ast::with_program(Path::new(file), source, |program, source| {
        let mut visitor = Scanner {
            file,
            source,
            policy,
            threshold,
            findings: Vec::new(),
        };
        visitor.visit_program(program);
        visitor.findings
    })
    .unwrap_or_default()
}

struct Scanner<'a> {
    file: &'a str,
    source: &'a str,
    policy: &'a Compiled,
    threshold: u64,
    findings: Vec<RuleFinding>,
}

impl<'a> Visit<'a> for Scanner<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.note_array_from(call);
        self.note_each(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        self.note_new_array(expr);
        walk::walk_new_expression(self, expr);
    }
}

impl Scanner<'_> {
    fn note_array_from(&mut self, call: &CallExpression<'_>) {
        if call.optional || !is_named_member(&call.callee, "Array", "from") {
            return;
        }
        let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        let Expression::ObjectExpression(object) = unwrap_ts_wrappers(argument) else {
            return;
        };
        let Some(length) = length_expression(object) else {
            return;
        };
        self.note_cardinality("Array.from length", length);
    }

    fn note_new_array(&mut self, expr: &NewExpression<'_>) {
        let Expression::Identifier(name) = &expr.callee else {
            return;
        };
        if name.name.as_str() != "Array" || expr.arguments.len() != 1 {
            return;
        }
        let Some(argument) = expr.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        self.note_cardinality("new Array length", unwrap_ts_wrappers(argument));
    }

    fn note_each(&mut self, call: &CallExpression<'_>) {
        if call.optional {
            return;
        }
        let Some(label) = self.each_label(&call.callee) else {
            return;
        };
        let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(argument) else {
            return;
        };
        let Some(numbers) = numeric_list(array) else {
            return;
        };
        for (value, span) in numbers {
            self.note_value(&format!("{label} size"), value, span);
        }
    }

    fn each_label(&self, callee: &Expression) -> Option<String> {
        let (object, property) = member_names(callee)?;
        self.policy
            .each_calls
            .iter()
            .any(|(configured_object, configured_property)| {
                configured_object == object && configured_property == property
            })
            .then(|| format!("{object}.{property}"))
    }

    fn note_cardinality(&mut self, kind: &str, expr: &Expression) {
        let Expression::NumericLiteral(literal) = expr else {
            return;
        };
        self.note_value(kind, literal.value, literal.span);
    }

    fn note_value(&mut self, kind: &str, value: f64, span: Span) {
        let Some(cardinality) = integer_cardinality(value) else {
            return;
        };
        if cardinality <= self.threshold || self.policy.exemptions.contains(&cardinality) {
            return;
        }
        let threshold = self.threshold;
        let line = byte_offset_to_line(self.source, span.start as usize) as usize;
        self.findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: self.file.to_string(),
            line,
            message: format!("{kind} {cardinality} exceeds threshold {threshold}"),
            import: None,
            target: None,
            causes: Vec::new(),
        });
    }
}
