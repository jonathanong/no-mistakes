#[derive(Clone)]
pub(super) enum Seg {
    Key(String),
    Idx(usize),
}

struct Line<'a> {
    indent: usize,
    text: &'a str,
    number: usize,
}

// A `- key: v` line becomes a `-` marker plus an inline `key: v` line.
fn flatten(source: &str) -> Vec<Line<'_>> {
    let mut out = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let mut text = raw.split(" #").next().unwrap_or(raw).trim_end();
        let mut indent = raw.len() - raw.trim_start().len();
        text = text.trim_start();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        while let Some(rest) = text
            .strip_prefix('-')
            .filter(|r| r.is_empty() || r.starts_with(' '))
        {
            out.push(Line {
                indent,
                text: "-",
                number: index + 1,
            });
            let inner = rest.trim_start();
            indent += 1 + (rest.len() - inner.len());
            text = inner;
        }
        if !text.is_empty() {
            out.push(Line {
                indent,
                text,
                number: index + 1,
            });
        }
    }
    out
}

fn key_of(text: &str) -> Option<&str> {
    let quote = text.chars().next().filter(|c| matches!(c, '"' | '\''));
    let Some(quote) = quote else {
        return text.split_once(':').map(|(key, _)| key.trim_end());
    };
    let inner = &text[1..];
    let end = inner.find(quote)?;
    inner[end + 1..]
        .trim_start()
        .starts_with(':')
        .then_some(&inner[..end])
}

fn range_end(lines: &[Line<'_>], at: usize, hi: usize, dash_ok: bool) -> usize {
    let indent = lines[at].indent;
    (at + 1..hi)
        .find(|&j| {
            let line = &lines[j];
            line.indent < indent || (line.indent == indent && !(dash_ok && line.text == "-"))
        })
        .unwrap_or(hi)
}

// Returns the deepest line that matches; a flow-style or missing child falls back to its parent.
pub(super) fn locate(source: &str, path: &[Seg]) -> usize {
    let lines = flatten(source);
    let (mut lo, mut hi) = (0, lines.len());
    let mut found = 1;
    for seg in path {
        if lo >= hi {
            break;
        }
        let indent = lines[lo].indent;
        let mut dashes = 0;
        let mut hit = None;
        for at in (lo..hi).filter(|&at| lines[at].indent == indent) {
            let text = lines[at].text;
            let matched = match seg {
                Seg::Key(key) => key_of(text) == Some(key.as_str()),
                Seg::Idx(want) => {
                    let is_dash = text == "-";
                    dashes += usize::from(is_dash);
                    is_dash && dashes - 1 == *want
                }
            };
            if matched {
                hit = Some(at);
                break;
            }
        }
        let Some(at) = hit else {
            break;
        };
        found = lines[at].number;
        lo = at + 1;
        hi = range_end(&lines, at, hi, matches!(seg, Seg::Key(_)));
    }
    found
}
