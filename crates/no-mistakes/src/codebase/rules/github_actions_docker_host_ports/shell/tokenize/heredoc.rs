use super::Tok;

pub(super) fn delimiter(chars: &[char], mut index: usize) -> (String, usize) {
    while matches!(chars.get(index), Some(' ' | '\t')) {
        index += 1;
    }
    let mut delim = String::new();
    while let Some(ch) = chars.get(index).copied() {
        if ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '<' | '>' | '(' | ')') {
            break;
        }
        index += 1;
        if ch == '\\' || ch == '\'' || ch == '"' {
            continue;
        }
        delim.push(ch);
    }
    (delim, index)
}

impl<'a> Tok<'a> {
    pub(super) fn skip_pending_bodies(&mut self) {
        while !self.pending_heredocs.is_empty() {
            let (delim, strip_tabs) = self.pending_heredocs.remove(0);
            if !self.skip_one_body(&delim, strip_tabs) {
                self.index = self.chars.len();
                self.pending_heredocs.clear();
                return;
            }
        }
    }

    fn skip_one_body(&mut self, delim: &str, strip_tabs: bool) -> bool {
        while self.index < self.chars.len() {
            let start = self.index;
            while self.index < self.chars.len() && self.chars[self.index] != '\n' {
                self.index += 1;
            }
            let text: String = self.chars[start..self.index].iter().collect();
            let compare = if strip_tabs {
                text.trim_start_matches('\t')
            } else {
                text.as_str()
            };
            let done = compare == delim;
            if self.index < self.chars.len() {
                self.index += 1;
                self.line += 1;
            }
            if done {
                return true;
            }
        }
        false
    }
}
