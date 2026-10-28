/// A content line outside comments and block scalar bodies.
pub(super) struct Entry<'a> {
    pub(super) line: usize,
    pub(super) indent: usize,
    /// Column where the mapping key starts, after any `- ` markers.
    pub(super) col: usize,
    /// Columns of each leading `- ` marker, so `- - k: 1` has two.
    pub(super) marks: Vec<usize>,
    pub(super) text: &'a str,
}

impl Entry<'_> {
    pub(super) fn dash(&self) -> bool {
        !self.marks.is_empty()
    }
}

pub(super) fn entries(source: &str) -> Vec<Entry<'_>> {
    let mut out: Vec<Entry<'_>> = Vec::new();
    let mut block: Option<usize> = None;
    for (index, raw) in source.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = raw.len() - trimmed.len();
        if block.is_some_and(|col| indent > col) {
            continue;
        }
        block = None;
        if trimmed.starts_with('#') {
            continue;
        }
        let mut col = indent;
        let mut text = trimmed;
        let mut marks = Vec::new();
        while text == "-" || text.starts_with("- ") {
            marks.push(col);
            let rest = text[1..].trim_start();
            col += text.len() - rest.len();
            text = rest;
        }
        if opens_block_scalar(text) {
            block = Some(col);
        }
        out.push(Entry {
            line: index + 1,
            indent,
            col,
            marks,
            text,
        });
    }
    out
}

fn opens_block_scalar(text: &str) -> bool {
    // A trailing ` # comment` may follow the header indicators.
    let text = text.split(" #").next().unwrap_or(text);
    let value = text.rsplit_once(": ").map_or(text, |(_, value)| value);
    let mut chars = value.trim_end().chars();
    matches!(chars.next(), Some('|' | '>'))
        && chars.all(|c| c == '+' || c == '-' || c.is_ascii_digit())
}
