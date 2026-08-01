use anyhow::Result;
#[cfg(feature = "mermaid-validation")]
use merman_analysis::Analyzer;
#[cfg(feature = "mermaid-validation")]
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
#[cfg(feature = "mermaid-validation")]
use crate::mermaid_validation::{
    validate_mermaid_fences, MermaidValidationDiagnostic, MermaidValidationDiagnosticCode,
};

pub const RULE_ID: &str = "markdown-mermaid-validation";

#[cfg(feature = "mermaid-validation")]
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

#[cfg(not(feature = "mermaid-validation"))]
pub fn check_with_files(
    _root: &Path,
    _config: &NoMistakesConfig,
    _files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    anyhow::bail!(feature_disabled_message())
}

#[cfg(feature = "mermaid-validation")]
pub(crate) fn check_with_files_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let documents = super::markdown_scope::mermaid_document_files(files);
    let analyzer = Analyzer::new();
    let all = config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let targets =
                super::path_filter::filter_markdown_rule_files(root, config, rule, &documents)?;
            let findings = targets
                .par_iter()
                .map(|path| findings_for_path(root, path, facts, &analyzer))
                .collect::<Result<Vec<_>>>()?;
            Ok(findings.into_iter().flatten().collect())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut findings = all.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(not(feature = "mermaid-validation"))]
pub(crate) fn check_with_files_and_facts(
    _root: &Path,
    _config: &NoMistakesConfig,
    _files: &[PathBuf],
    _facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    anyhow::bail!(feature_disabled_message())
}

#[cfg(not(feature = "mermaid-validation"))]
fn feature_disabled_message() -> &'static str {
    "markdown-mermaid-validation requires the default mermaid-validation Cargo feature; this feature gate lets core and benchmark builds exclude the merman-analysis dependency; rebuild without --no-default-features or enable --features mermaid-validation"
}

#[cfg(all(test, not(feature = "mermaid-validation")))]
mod feature_disabled_tests {
    #[test]
    fn message_explains_the_feature_boundary_and_fix() {
        let message = super::feature_disabled_message();
        assert!(message.contains("core and benchmark builds"));
        assert!(message.contains("exclude the merman-analysis dependency"));
        assert!(message.contains("--features mermaid-validation"));
    }
}

#[cfg(feature = "mermaid-validation")]
fn findings_for_path(
    root: &Path,
    path: &Path,
    facts: &super::markdown_facts::MarkdownFactMap,
    analyzer: &Analyzer,
) -> Result<Vec<RuleFinding>> {
    let markdown = facts.get_for_rule(path, RULE_ID)?;
    let file = super::markdown_scope::finding_key(root, path);
    Ok(
        validate_mermaid_fences(analyzer, &markdown.mermaid_fences, &file)
            .diagnostics
            .into_iter()
            .map(finding)
            .collect(),
    )
}

#[cfg(feature = "mermaid-validation")]
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

#[cfg(all(test, feature = "mermaid-validation"))]
#[path = "markdown_mermaid_validation/tests.rs"]
mod tests;
