use super::MdxExpressionScanner;

pub(super) fn is_jsx_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$' | b'>')
}

impl MdxExpressionScanner {
    pub(super) fn observe_jsx_byte(&mut self, byte: u8, next: Option<u8>) -> bool {
        if let Some(delimiter) = self.jsx_quote {
            if byte == delimiter {
                self.jsx_quote = None;
            }
            return true;
        }
        if self.jsx_expression_root_depth.is_some() {
            return self.observe_expression_jsx_byte(byte, next);
        }
        if self.depth > 0 || self.esm {
            if byte == b'<' && self.can_start_regex && next.is_some_and(is_jsx_start) {
                self.start_expression_jsx();
                return true;
            }
            return false;
        }
        self.observe_top_level_jsx_byte(byte, next)
    }

    fn observe_expression_jsx_byte(&mut self, byte: u8, next: Option<u8>) -> bool {
        if self.jsx_opening {
            return self.observe_jsx_tag_byte(byte, next);
        }
        if self.jsx_text {
            match (byte, next) {
                (b'<', Some(b'/')) => self.start_closing_tag(),
                (b'<', Some(next)) if is_jsx_start(next) => self.start_child_tag(),
                (b'{', _) => self.start_jsx_javascript(true),
                _ => {}
            }
            return true;
        }
        if byte == b'<' && self.can_start_regex && next.is_some_and(is_jsx_start) {
            self.jsx_return_to_js_depths.push(self.jsx_element_depth);
            self.start_child_tag();
            return true;
        }
        false
    }

    fn observe_top_level_jsx_byte(&mut self, byte: u8, next: Option<u8>) -> bool {
        if self.jsx_opening {
            return self.observe_jsx_tag_byte(byte, next);
        }
        if byte == b'<' && next.is_some_and(is_jsx_start) {
            self.jsx_opening = true;
        }
        false
    }

    fn observe_jsx_tag_byte(&mut self, byte: u8, next: Option<u8>) -> bool {
        match (byte, next) {
            (b'\'' | b'"', _) => self.jsx_quote = Some(byte),
            (b'{', _) => {
                self.start_jsx_javascript(false);
                return true;
            }
            (b'/', Some(b'>')) => self.jsx_self_closing_tag = true,
            (b'>', _) => self.finish_jsx_tag(),
            _ => {}
        }
        true
    }

    fn start_expression_jsx(&mut self) {
        self.jsx_expression_root_depth = Some(self.depth);
        self.jsx_element_depth = 1;
        self.jsx_opening = true;
        self.jsx_text = false;
    }

    fn start_child_tag(&mut self) {
        self.jsx_element_depth += 1;
        self.jsx_opening = true;
        self.jsx_closing_tag = false;
        self.jsx_text = false;
    }

    fn start_closing_tag(&mut self) {
        self.jsx_opening = true;
        self.jsx_closing_tag = true;
        self.jsx_text = false;
    }

    fn finish_jsx_tag(&mut self) {
        self.jsx_opening = false;
        if self.jsx_closing_tag || self.jsx_self_closing_tag {
            self.jsx_element_depth = self.jsx_element_depth.saturating_sub(1);
        }
        self.jsx_closing_tag = false;
        self.jsx_self_closing_tag = false;
        if self.jsx_element_depth == 0 {
            self.jsx_expression_root_depth = None;
            self.jsx_text = false;
        } else if self
            .jsx_return_to_js_depths
            .last()
            .is_some_and(|depth| *depth == self.jsx_element_depth)
        {
            self.jsx_return_to_js_depths.pop();
            self.jsx_text = false;
        } else {
            self.jsx_text = true;
        }
    }

    fn start_jsx_javascript(&mut self, resume_text: bool) {
        self.jsx_js_base_depth = Some(self.depth);
        self.jsx_js_resume_text = resume_text;
        self.depth += 1;
        self.can_start_regex = true;
        self.jsx_opening = false;
        self.jsx_text = false;
    }

    pub(super) fn resume_jsx_after_expression(&mut self) {
        if self.jsx_js_base_depth == Some(self.depth) {
            self.jsx_js_base_depth = None;
            self.jsx_text = self.jsx_js_resume_text;
            self.jsx_opening = !self.jsx_js_resume_text;
        }
    }
}
