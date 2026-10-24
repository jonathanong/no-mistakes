use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Meta};

pub const RULE_ID: &str = "rust-no-inline-allows";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::rust_rules_combined::standalone::check(root, config, RULE_ID)
}

pub(crate) fn findings_from_parsed(
    path: &Path,
    root: &Path,
    parsed: &syn::File,
) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let mut visitor = AllowAttrVisitor::default();
    visitor.visit_file(parsed);
    visitor
        .findings
        .into_iter()
        .map(|finding| RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: finding.line,
            message: format!(
                "inline #[allow({})] attribute - fix the warning or move test-only code to a sibling test module",
                finding.lints
            ),
            import: None,
            target: None,
        })
        .collect()
}

#[derive(Default)]
struct AllowAttrVisitor {
    findings: Vec<AllowAttrFinding>,
}

struct AllowAttrFinding {
    line: usize,
    lints: String,
}

impl<'ast> Visit<'ast> for AllowAttrVisitor {
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if attr.path().is_ident("allow") {
            self.findings.push(AllowAttrFinding {
                line: attr.span().start().line,
                lints: allow_lints(attr),
            });
        }
        visit::visit_attribute(self, attr);
    }
}

fn allow_lints(attr: &Attribute) -> String {
    match &attr.meta {
        Meta::List(list) => list
            .tokens
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
