//! Reproduce Oxc 0.153's diagnostic ordering when restoring its fatal result.
use super::ParserReturn;
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_span::Span;

pub(super) fn diagnostics(parsed: &mut ParserReturn<'_>, rejected: Span, trivia_end: usize) {
    let flow = parsed
        .program
        .source_type
        .is_javascript()
        .then(|| {
            parsed
                .program
                .comments
                .first()
                .filter(|comment| {
                    comment
                        .span
                        .source_text(parsed.program.source_text)
                        .contains("@flow")
                })
                .map(|comment| comment.span)
        })
        .flatten();
    // The old parser suppressed its fatal diagnostic if lexing the next token
    // reached EOF with a lexer error. Preserve those existing lexer diagnostics.
    let eof_lookahead = parsed.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.message.as_ref(),
            "Unterminated multiline comment" | "Unterminated string"
        ) && diagnostic.labels.iter().any(|label| {
            label.offset() as usize == trivia_end
                && (label.offset() + label.len()) as usize == parsed.program.source_text.len()
        })
    });
    let original = std::mem::take(&mut parsed.diagnostics);
    let mut deferred = Diagnostics::new();
    for diagnostic in original {
        // Oxc appends deferred HTML-comment errors after fatal/Flow selection.
        if diagnostic.message == "HTML comments are not allowed in modules" {
            if diagnostic
                .labels
                .iter()
                .all(|label| (label.offset() as usize) < trivia_end)
            {
                deferred.push(diagnostic);
            }
        } else if diagnostic.labels.iter().all(|label| {
            label.offset() < rejected.start && label.offset() + label.len() <= rejected.start
        }) || (eof_lookahead
            && diagnostic
                .labels
                .iter()
                .all(|label| label.offset() as usize >= trivia_end))
        {
            parsed.diagnostics.push(diagnostic);
        }
    }
    parsed.is_flow_language = flow.is_some();
    if let Some(span) = flow {
        parsed.diagnostics.clear();
        parsed
            .diagnostics
            .push(OxcDiagnostic::error("Flow is not supported").with_label(span));
    } else if !eof_lookahead {
        parsed.diagnostics.push(
            OxcDiagnostic::error("An export name cannot include a unicode lone surrogate")
                .with_label(rejected),
        );
    }
    parsed.diagnostics.extend(deferred);
}
