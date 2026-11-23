use super::super::super::url::skip_comment;
use super::Scan;

impl Scan<'_> {
    /// `#` starts a comment only at the beginning of a word, outside quotes.
    /// The newline stays for `separator`, so a here-document opener on the
    /// same line still skips its body (`cat <<EOF # note`).
    pub(super) fn comment(&mut self, ch: char) -> bool {
        if ch != '#' || self.quoting.is_some() || self.word_quoted || !self.word.is_empty() {
            return false;
        }
        let start = self.index;
        while self.index < self.chars.len() && self.chars[self.index] != '\n' {
            // A backtick can close the substitution before its comment line ends.
            if self.in_backticks && self.chars[self.index] == '`' && !self.escaped() {
                break;
            }
            self.index += 1;
        }
        if self.collect_skips
            && !self.in_backticks
            && !start
                .checked_sub(1)
                .is_some_and(|index| self.chars[index] != '\n' && self.escaped_at(index))
            && skip_comment(&self.chars[start + 1..self.index].iter().collect::<String>())
        {
            let standalone = self.chars[..start]
                .iter()
                .rev()
                .take_while(|ch| **ch != '\n')
                .all(|ch| ch.is_whitespace());
            self.skip_lines.push(self.line + usize::from(standalone));
        }
        true
    }
}
