use super::MdxExpressionScanner;

pub(super) enum LeadingContinuation {
    Continue,
    Defer,
    End,
}

impl MdxExpressionScanner {
    pub(super) fn resolve_deferred_esm(&mut self, line: &[u8]) {
        if !self.esm_line_complete {
            return;
        }
        match leading_esm_continuation(line) {
            LeadingContinuation::Continue => self.esm_line_complete = false,
            LeadingContinuation::Defer => {}
            LeadingContinuation::End => self.close_esm(),
        }
    }

    pub(crate) fn resolve_deferred_esm_before_markdown(&mut self, source: &str, start: usize) {
        let line_end = source.as_bytes()[start..]
            .iter()
            .position(|byte| matches!(byte, b'\r' | b'\n'))
            .map_or(source.len(), |offset| start + offset);
        self.resolve_deferred_esm(&source.as_bytes()[start..line_end]);
    }

    pub(super) fn close_esm(&mut self) {
        self.esm = false;
        self.esm_line_complete = false;
        self.last_js_code_byte = None;
        self.esm_export_prefix = false;
        self.esm_value_pending = false;
    }
}

pub(super) fn leading_esm_continuation(line: &[u8]) -> LeadingContinuation {
    let line = &line[line
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()..];
    if line.is_empty() || line.starts_with(b"//") || line.starts_with(b"/*") {
        return LeadingContinuation::Defer;
    }
    let backticks = line.iter().take_while(|byte| **byte == b'`').count();
    let tagged_template = (1..=2).contains(&backticks);
    if tagged_template || b".?([+-*/%&|^<>=,".contains(&line[0]) {
        LeadingContinuation::Continue
    } else {
        LeadingContinuation::End
    }
}
