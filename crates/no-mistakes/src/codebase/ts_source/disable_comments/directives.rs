#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DisableDirective {
    File { line: u32 },
    Line { line: u32 },
    NextLine { line: u32 },
}

/// Returns exact provenance for a supported suppression directive.
///
/// This is the single directive parser used by both filtering and the
/// aggregate check audit report, so accounting cannot fabricate a line number.
pub fn matching_disable_directive(
    source: &str,
    finding_line: Option<u32>,
    rule_id: &str,
) -> Option<DisableDirective> {
    if let Some(line) = super::disable_file_directive_line(source, rule_id) {
        return Some(DisableDirective::File { line });
    }
    let line = finding_line?;
    if super::has_disable_comment(source, line, rule_id) {
        return Some(DisableDirective::NextLine {
            line: line.saturating_sub(1),
        });
    }
    super::has_disable_line_comment(source, line, rule_id)
        .then_some(DisableDirective::Line { line })
}
