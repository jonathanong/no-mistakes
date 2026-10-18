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
    matching_disable_directive_tracking(source, finding_line, rule_id, true)
}

pub(crate) fn matching_disable_directive_tracking(
    source: &str,
    finding_line: Option<u32>,
    rule_id: &str,
    track_quotes: bool,
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
    has_disable_line_comment_tracking(source, line, rule_id, track_quotes)
        .then_some(DisableDirective::Line { line })
}

pub(crate) fn has_disable_line_comment_tracking(
    source: &str,
    stmt_line: u32,
    rule_id: &str,
    track_quotes: bool,
) -> bool {
    stmt_line != 0
        && line_comment_directive_matches(
            source,
            stmt_line,
            "no-mistakes-disable-line ",
            rule_id,
            track_quotes,
        )
}

fn line_comment_directive_matches(
    source: &str,
    stmt_line: u32,
    directive: &str,
    rule_id: &str,
    track_quotes: bool,
) -> bool {
    let mut state = super::LineCommentScanState {
        track_quotes,
        ..super::LineCommentScanState::default()
    };
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let line_number = (index + 1) as u32;
        let comment = super::line_comment_start(line, &mut state);
        if line_number != stmt_line {
            continue;
        }
        let Some((comment_start, prefix_len)) = comment else {
            return false;
        };
        let rest = super::comment_body(line, comment_start, prefix_len);
        let Some(after_directive) = rest.strip_prefix(directive) else {
            return false;
        };
        return super::rule_part_matches(after_directive.trim(), rule_id);
    }
    false
}
