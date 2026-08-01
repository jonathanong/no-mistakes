use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::mermaid_validation::{
    validate_mermaid_fences, MermaidValidationDiagnostic, MermaidValidationDiagnosticCode,
};

pub const RULE_ID: &str = "markdown-mermaid-validation";

pub fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    let mut plan = super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(super::markdown_scope::mermaid_document_files(files));
    let facts = super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    check_with_files_and_facts(root, config, files, &facts)
}

pub(crate) fn check_with_files_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let documents = super::markdown_scope::mermaid_document_files(files);
    let all = config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let targets =
                super::path_filter::filter_markdown_rule_files(root, config, rule, &documents)?;
            Ok(targets
                .par_iter()
                .flat_map(|path| findings_for_path(root, path, facts))
                .collect())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut findings = all.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn findings_for_path(
    root: &Path,
    path: &Path,
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Vec<RuleFinding> {
    let Some(markdown) = facts.get(path) else {
        return Vec::new();
    };
    let file = super::markdown_scope::finding_key(root, path);
    validate_mermaid_fences(&markdown.mermaid_fences, &file)
        .diagnostics
        .into_iter()
        .map(finding)
        .collect()
}

fn finding(diagnostic: MermaidValidationDiagnostic) -> RuleFinding {
    let message = match diagnostic.code {
        MermaidValidationDiagnosticCode::InvalidSyntax => {
            let diagram_type = diagnostic
                .diagram_type
                .as_deref()
                .map(|kind| format!(" ({kind})"))
                .unwrap_or_default();
            let location = match (diagnostic.diagram_line, diagnostic.diagram_column) {
                (Some(line), Some(column)) => format!(" at diagram line {line}, column {column}"),
                (Some(line), None) => format!(" at diagram line {line}"),
                _ => String::new(),
            };
            format!(
                "invalid Mermaid diagram{diagram_type}{location}: {}",
                diagnostic.message
            )
        }
        MermaidValidationDiagnosticCode::UnclosedFence => diagnostic.message,
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: diagnostic.file,
        line: diagnostic.fence_line,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
#[path = "markdown_mermaid_validation/tests.rs"]
mod tests;
