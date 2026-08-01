use merman_analysis::{Analyzer, DiagnosticSeverity};
use serde::Serialize;

mod extract;
pub(crate) use extract::{
    extract_mermaid_fences, extract_mermaid_fences_with_mdx_html_fallback, MermaidFence,
    MermaidFenceCollector,
};

const DEFAULT_FILE: &str = "<input>";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MermaidValidationDiagnosticCode {
    InvalidSyntax,
    UnclosedFence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MermaidValidationDiagnostic {
    pub code: MermaidValidationDiagnosticCode,
    pub file: String,
    pub fence_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagram_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagram_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagram_type: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MermaidValidationResult {
    pub valid: bool,
    pub diagram_count: usize,
    pub diagnostics: Vec<MermaidValidationDiagnostic>,
}

pub(crate) fn validate_mermaid_fences(
    fences: &[MermaidFence],
    file: &str,
) -> MermaidValidationResult {
    let analyzer = Analyzer::new();
    let mut diagnostics = Vec::new();

    for fence in fences {
        if !fence.closed {
            diagnostics.push(MermaidValidationDiagnostic {
                code: MermaidValidationDiagnosticCode::UnclosedFence,
                file: file.to_string(),
                fence_line: fence.fence_line,
                diagram_line: None,
                diagram_column: None,
                diagram_type: None,
                message: "unclosed Mermaid fence (missing closing fence)".to_string(),
            });
            continue;
        }

        diagnostics.extend(
            analyzer
                .analyze(&fence.content)
                .diagnostics
                .into_iter()
                .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
                .map(|diagnostic| MermaidValidationDiagnostic {
                    code: MermaidValidationDiagnosticCode::InvalidSyntax,
                    file: file.to_string(),
                    fence_line: fence.fence_line,
                    diagram_line: diagnostic.span.as_ref().map(|span| span.line),
                    diagram_column: diagnostic.span.as_ref().map(|span| span.column),
                    diagram_type: diagnostic.diagram_type,
                    message: diagnostic.message,
                }),
        );
    }

    MermaidValidationResult {
        valid: diagnostics.is_empty(),
        diagram_count: fences.len(),
        diagnostics,
    }
}

/// Validate every fenced Mermaid diagram in a Markdown document.
pub fn validate_markdown(content: &str, file: Option<&str>) -> MermaidValidationResult {
    let is_mdx = file.is_some_and(|file| {
        let path = file.split(['?', '#']).next().unwrap_or(file);
        std::path::Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mdx"))
    });
    let fences = if is_mdx {
        extract_mermaid_fences_with_mdx_html_fallback(content)
    } else {
        extract_mermaid_fences(content)
    };
    validate_mermaid_fences(&fences, file.unwrap_or(DEFAULT_FILE))
}

#[cfg(test)]
#[path = "mermaid_validation/tests.rs"]
mod tests;
