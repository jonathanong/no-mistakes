use super::*;
use anyhow::Context;

pub(super) fn parse_program(
    path: &Path,
    source: &str,
    mode: ParseMode,
) -> Result<CachedProgram, String> {
    let source_type = source_type(path, mode)
        .with_context(|| format!("unsupported JavaScript/TypeScript file: {}", path.display()))
        .map_err(|error| error.to_string())?;
    let owner = ProgramOwner {
        allocator: Allocator::default(),
        source: source.to_string(),
        source_type,
    };
    CachedProgram::try_new(owner, |owner| {
        let parsed = crate::ast::parse(path, &owner.allocator, &owner.source, owner.source_type);
        let strict_error = if parsed.panicked || !parsed.diagnostics.is_empty() {
            Some(
                parsed
                    .diagnostics
                    .first()
                    .map(|error| format!("{error:?}"))
                    .unwrap_or("unknown error (parser panicked)".to_string()),
            )
        } else {
            None
        };
        let diagnostic_error = (parsed.panicked || !parsed.diagnostics.is_empty()).then(|| {
            crate::codebase::ts_source::format_parse_diagnostic(path, &parsed.diagnostics)
        });
        let panic_error = parsed.panicked.then(|| {
            let detail = parsed
                .diagnostics
                .first()
                .map(|error| format!("{error:?}"))
                .unwrap_or("unknown error (parser panicked)".to_string());
            format!("failed to parse TypeScript source: {detail}")
        });
        Ok(ParsedProgram {
            program: parsed.program,
            strict_error,
            diagnostic_error,
            panic_error,
        })
    })
}

fn source_type(path: &Path, mode: ParseMode) -> anyhow::Result<SourceType> {
    match mode {
        ParseMode::Standard => SourceType::from_path(path).map_err(Into::into),
        ParseMode::TypeScriptFallback => SourceType::from_path(path).or(Ok(SourceType::ts())),
        ParseMode::LegacySymbols => {
            let is_tsx = matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("tsx" | "jsx")
            );
            Ok(if is_tsx {
                SourceType::tsx()
            } else {
                SourceType::ts()
            })
        }
    }
}
