use oxc_diagnostics::OxcDiagnostic;
use std::path::Path;

/// Format the first parser diagnostic, retaining the panic fallback used by
/// callers when the parser aborts without providing diagnostic details.
pub(crate) fn format_parse_diagnostic(path: &Path, diagnostics: &[OxcDiagnostic]) -> String {
    match diagnostics.first() {
        Some(error) => format!("parsing {}: {error:?}", path.display()),
        None => format!(
            "parsing {}: parser panicked without diagnostic details",
            path.display()
        ),
    }
}
