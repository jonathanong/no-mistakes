use super::literals::{
    count_oversized_table_cells, integer_cardinality, is_named_member, length_expression,
    member_names, numeric_list,
};
use super::{in_scope, is_script, relative_file, Compiled, EachTableMode, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers, SourceStore};
use oxc_ast::ast::{Argument, CallExpression, Expression, NewExpression};
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
        self.note_array_call(call);
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
        self.note_array_constructor(&expr.callee, &expr.arguments);
    }

    fn note_array_call(&mut self, call: &CallExpression<'_>) {
        if call.optional {
            return;
        }
        self.note_array_constructor(&call.callee, &call.arguments);
    }

    fn note_array_constructor(&mut self, callee: &Expression<'_>, arguments: &[Argument<'_>]) {
        let Expression::Identifier(name) = callee else {
            return;
        };
        if name.name.as_str() != "Array" || arguments.len() != 1 {
            return;
        }
        let Some(argument) = arguments.first().and_then(|arg| arg.as_expression()) else {
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
        match self.policy.each_table_mode {
            EachTableMode::PerElement => {
                let kind = format!("{label} size");
                for (value, span) in numeric_list(array) {
                    self.note_value(&kind, value, span);
                }
            }
            EachTableMode::AllElements => {
                let Some(count) = count_oversized_table_cells(array, |value| {
                    integer_cardinality(value).is_some_and(|size| self.is_oversized(size))
                }) else {
                    return;
                };
                if count > 0 {
                    let threshold = self.threshold;
                    self.note_finding(
                        format!(
                            "{label} table has {count} sizes all exceeding threshold {threshold}"
                        ),
                        array.span,
                    );
                }
            }
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
        if self.is_oversized(cardinality) {
            let threshold = self.threshold;
            self.note_finding(
                format!("{kind} {cardinality} exceeds threshold {threshold}"),
                span,
            );
        }
    }

    fn is_oversized(&self, cardinality: u64) -> bool {
        cardinality > self.threshold && !self.policy.exemptions.contains(&cardinality)
    }

    fn note_finding(&mut self, message: String, span: Span) {
        let line = byte_offset_to_line(self.source, span.start as usize) as usize;
        self.findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: self.file.to_string(),
            line,
            message,
            import: None,
            target: None,
            causes: Vec::new(),
        });
    }
}
