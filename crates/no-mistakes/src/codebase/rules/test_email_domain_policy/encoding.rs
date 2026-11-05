use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

pub(super) struct DecodedLine<'a> {
    pub(super) text: Cow<'a, str>,
    spans: Vec<Range<usize>>,
}

impl DecodedLine<'_> {
    pub(super) fn original<'a>(&self, source: &'a str, range: Range<usize>) -> &'a str {
        if self.spans.is_empty() {
            &source[range]
        } else {
            &source[self.spans[range.start].start..self.spans[range.end - 1].end]
        }
    }
}

pub(super) fn decode(source: &str) -> DecodedLine<'_> {
    static ENTITY: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"&(?:#[xX][0-9a-fA-F]+;?|#[0-9]+;?|[A-Za-z][A-Za-z0-9]+;)")
            .expect("HTML entity regex")
    });
    if !source.contains('&') {
        return DecodedLine {
            text: Cow::Borrowed(source),
            spans: Vec::new(),
        };
    }
    let mut text = String::new();
    let mut spans = Vec::new();
    let mut offset = 0;
    for entity in ENTITY.find_iter(source) {
        append_literal(&mut text, &mut spans, source, offset..entity.start());
        let decoded = htmlize::unescape(entity.as_str());
        text.push_str(&decoded);
        spans.extend(std::iter::repeat_n(entity.range(), decoded.len()));
        offset = entity.end();
    }
    append_literal(&mut text, &mut spans, source, offset..source.len());
    DecodedLine {
        text: Cow::Owned(text),
        spans,
    }
}

fn append_literal(
    text: &mut String,
    spans: &mut Vec<Range<usize>>,
    source: &str,
    range: Range<usize>,
) {
    text.push_str(&source[range.clone()]);
    spans.extend(range.map(|index| index..index + 1));
}
