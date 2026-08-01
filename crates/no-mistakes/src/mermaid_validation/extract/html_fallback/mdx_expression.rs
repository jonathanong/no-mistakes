#[derive(Clone, Copy, Default)]
enum Literal {
    #[default]
    None,
    SingleQuoted,
    DoubleQuoted,
    Template,
    Regex,
    BlockComment,
}

#[path = "mdx_expression/javascript.rs"]
mod javascript;
use javascript::ByteAction;
#[path = "mdx_expression/jsx.rs"]
mod jsx;

#[derive(Default)]
pub(crate) struct MdxExpressionScanner {
    depth: usize,
    esm: bool,
    paren_depth: usize,
    bracket_depth: usize,
    literal: Literal,
    escaped: bool,
    regex_character_class: bool,
    can_start_regex: bool,
    jsx_opening: bool,
    jsx_quote: Option<u8>,
    jsx_expression_root_depth: Option<usize>,
    jsx_element_depth: usize,
    jsx_closing_tag: bool,
    jsx_self_closing_tag: bool,
    jsx_text: bool,
    jsx_js_base_depth: Option<usize>,
    jsx_js_resume_text: bool,
    jsx_return_to_js_depths: Vec<usize>,
    last_js_code_byte: Option<u8>,
    esm_export_prefix: bool,
    esm_value_pending: bool,
}

impl MdxExpressionScanner {
    pub(crate) fn is_inside_expression(&self) -> bool {
        self.depth > 0 || self.esm
    }

    pub(crate) fn is_masking_markdown(&self) -> bool {
        self.is_inside_expression() || self.jsx_quote.is_some()
    }

    pub(super) fn observe_line(&mut self, line: &[u8]) {
        self.observe_line_until_closed(line, false);
    }

    fn observe_line_until_closed(&mut self, line: &[u8], stop_when_closed: bool) -> bool {
        if !self.is_masking_markdown() && starts_esm_statement(line) {
            self.esm = true;
            self.can_start_regex = true;
            self.last_js_code_byte = None;
            self.esm_export_prefix = false;
            self.esm_value_pending = false;
        }
        let mut index = 0;
        while index < line.len() {
            let byte = line[index];
            let next = line.get(index + 1).copied();
            if self.observe_jsx_byte(byte, next) {
                index += 1;
                continue;
            }
            if !self.is_inside_expression() {
                if let Some(end) = markdown_escape_end(line, index) {
                    index = end;
                    continue;
                }
            }
            if let Some(end) = self.javascript_token_end(line, index) {
                index = end;
                continue;
            }
            match self.observe_javascript_byte(byte, next) {
                ByteAction::Advance(extra) => index += extra + 1,
                ByteAction::Break => break,
            }
            if stop_when_closed && !self.is_masking_markdown() && !self.jsx_opening {
                if let Some(next) = next_mdx_region_start(line, index) {
                    index = next;
                } else {
                    self.escaped = false;
                    return true;
                }
            }
        }
        if matches!(self.literal, Literal::Regex) {
            self.literal = Literal::None;
            self.regex_character_class = false;
        }
        if self.esm
            && self.depth == 0
            && self.paren_depth == 0
            && self.bracket_depth == 0
            && matches!(self.literal, Literal::None)
            && !esm_token_continues(self.last_js_code_byte)
            && !self.esm_export_prefix
            && !self.esm_value_pending
        {
            self.esm = false;
            self.last_js_code_byte = None;
            self.esm_export_prefix = false;
            self.esm_value_pending = false;
        }
        self.escaped = false;
        false
    }

    pub(crate) fn observe_source(&mut self, source: &[u8]) {
        for line in source.split(|byte| matches!(byte, b'\r' | b'\n')) {
            let stop_when_closed = self.is_masking_markdown();
            self.observe_line_until_closed(line, stop_when_closed);
        }
    }
}

fn starts_esm_statement(line: &[u8]) -> bool {
    let line = line.strip_prefix(b"\xef\xbb\xbf").unwrap_or(line);
    let line = &line[line
        .iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()..];
    line.strip_prefix(b"import")
        .is_some_and(|rest| esm_declaration_follows(rest, true))
        || line
            .strip_prefix(b"export")
            .is_some_and(|rest| esm_declaration_follows(rest, false))
}

fn esm_declaration_follows(rest: &[u8], import: bool) -> bool {
    rest.first().is_none_or(|byte| {
        byte.is_ascii_whitespace()
            || matches!(byte, b'{' | b'*')
            || (import && matches!(byte, b'\'' | b'"'))
            || rest.starts_with(b"//")
            || rest.starts_with(b"/*")
    })
}

fn esm_token_continues(byte: Option<u8>) -> bool {
    byte.is_some_and(|byte| b"=,.:?!+-*/%&|^<>".contains(&byte))
}

fn markdown_escape_end(line: &[u8], start: usize) -> Option<usize> {
    if line.get(start) != Some(&b'\\') {
        return None;
    }
    let count = line[start..]
        .iter()
        .take_while(|byte| **byte == b'\\')
        .count();
    let after_slashes = start + count;
    Some(
        if count % 2 == 1 && line.get(after_slashes) == Some(&b'{') {
            after_slashes + 1
        } else {
            after_slashes
        },
    )
}

fn next_mdx_region_start(line: &[u8], mut index: usize) -> Option<usize> {
    while index < line.len() {
        match line[index] {
            b'\\' => index = markdown_escape_end(line, index)?,
            b'{' => return Some(index),
            b'<' if line.get(index + 1).copied().is_some_and(jsx::is_jsx_start) => {
                return Some(index);
            }
            b'`' => {
                let length = line[index..]
                    .iter()
                    .take_while(|byte| **byte == b'`')
                    .count();
                let closer = line[index + length..]
                    .windows(length)
                    .position(|window| window.iter().all(|byte| *byte == b'`'));
                let closer = closer?;
                index += length + closer + length;
            }
            _ => index += 1,
        }
    }
    None
}

#[cfg(test)]
#[path = "mdx_expression/tests.rs"]
mod tests;
