use super::callee::is_candidate_url;
use super::normalize::normalize_url_pattern;
use super::regex_sample::regex_path_sample;
use super::statics::static_zero_arg_path_call;
use crate::playwright::ast;
use oxc_ast::ast::{Argument, BinaryExpression, CallExpression};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;

/// Parse `a[href="/users/42"]` to `/users/42`.
pub fn extract_href_from_selector(selector: &str) -> Option<String> {
    let quoted = selector
        .split("href=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next());
    let single_quoted = selector
        .split("href='")
        .nth(1)
        .and_then(|rest| rest.split('\'').next());
    let url = quoted.or(single_quoted)?;
    if is_candidate_url(url) {
        Some(url.to_string())
    } else {
        None
    }
}

pub(super) struct LiteralVisitor<'a> {
    pub source: &'a str,
    pub static_zero_arg_paths: &'a HashMap<String, Vec<String>>,
    pub literals: Vec<String>,
}

impl<'a> Visit<'a> for LiteralVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let urls = static_zero_arg_path_call(call, self.static_zero_arg_paths);
        if !urls.is_empty() {
            self.literals.extend(urls);
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_string_literal(&mut self, literal: &oxc_ast::ast::StringLiteral<'a>) {
        self.literals.push(literal.value.to_string());
    }

    fn visit_reg_exp_literal(&mut self, literal: &oxc_ast::ast::RegExpLiteral<'a>) {
        if let Some(sample) = regex_path_sample(literal.regex.pattern.text.as_str()) {
            self.literals.push(sample);
        }
    }

    fn visit_template_literal(&mut self, template: &oxc_ast::ast::TemplateLiteral<'a>) {
        self.literals
            .push(ast::template_literal_text(template, self.source));
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        // Fold `'/users/' + userId` into `/users/${userId}` so an unresolved tail still
        // matches a dynamic route segment. Only when the fold yields a candidate path;
        // otherwise fall back to the default walk so unrelated `+` expressions (and any
        // string literals nested within them) keep their existing extraction behavior.
        if let Some(folded) = ast::binary_concat_path_text(binary, self.source) {
            if is_candidate_url(&folded) {
                self.literals.push(folded);
                return;
            }
        }
        walk::walk_binary_expression(self, binary);
    }
}

pub fn candidate_literals(
    arguments: &[Argument<'_>],
    source: &str,
    static_zero_arg_paths: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut visitor = LiteralVisitor {
        source,
        static_zero_arg_paths,
        literals: Vec::new(),
    };
    for argument in arguments {
        visitor.visit_argument(argument);
    }
    visitor
        .literals
        .into_iter()
        .filter(|url| is_candidate_url(url))
        .collect()
}

pub fn direct_url_pattern_literals(
    arguments: &[Argument<'_>],
    source: &str,
    static_zero_arg_paths: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut visitor = LiteralVisitor {
        source,
        static_zero_arg_paths,
        literals: Vec::new(),
    };
    for argument in arguments {
        visitor.visit_argument(argument);
    }
    visitor
        .literals
        .into_iter()
        .filter_map(|url| normalize_url_pattern(&url))
        .collect()
}

pub fn argument_literals(
    argument: &Argument<'_>,
    source: &str,
    static_zero_arg_paths: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    match argument {
        Argument::StringLiteral(literal) => vec![literal.value.to_string()],
        Argument::TemplateLiteral(template) => vec![ast::template_literal_text(template, source)],
        Argument::BinaryExpression(binary) => {
            // `page.goto('/users/' + id)` folds to `/users/${id}` (#391).
            ast::binary_concat_path_text(binary, source)
                .into_iter()
                .collect()
        }
        Argument::CallExpression(call) => static_zero_arg_path_call(call, static_zero_arg_paths),
        _ => Vec::new(),
    }
}

pub fn argument_candidate_literals(
    argument: &Argument<'_>,
    source: &str,
    static_zero_arg_paths: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    match argument {
        Argument::ObjectExpression(_) => Vec::new(),
        _ => candidate_literals(
            std::slice::from_ref(argument),
            source,
            static_zero_arg_paths,
        ),
    }
}

#[cfg(test)]
mod tests;
