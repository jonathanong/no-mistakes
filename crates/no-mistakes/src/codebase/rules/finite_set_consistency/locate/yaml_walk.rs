use super::entries::{entries, Entry};

fn key_matches(text: &str, key: &str) -> bool {
    let plain = text
        .strip_prefix(key)
        .and_then(|rest| rest.strip_prefix(':'))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace));
    // A quoted key ends at its closing quote, so `"a":[1]` needs no space.
    let quoted = ['"', '\''].iter().any(|quote| {
        text.strip_prefix(*quote)
            .and_then(|rest| rest.strip_prefix(key))
            .and_then(|rest| rest.strip_prefix(*quote))
            .is_some_and(|rest| rest.starts_with(':'))
    });
    plain || quoted
}

/// Entries owned by the previously matched segment.
#[derive(Clone, Copy)]
struct Scope<'a, 'b> {
    entries: &'b [Entry<'a>],
    /// How many `- ` markers of the first entry are already consumed. Zero
    /// means the first entry is an ordinary line, not an item line.
    consumed: usize,
}

pub(super) fn yaml_key_line(source: &str, path: &[String]) -> usize {
    let all = entries(source);
    // A JSON document opens with a bare brace; its keys are the next level.
    let start = usize::from(all.first().is_some_and(|first| first.text == "{"));
    let mut scope = Scope {
        entries: &all[start..],
        consumed: 0,
    };
    let mut found = 1;
    for segment in path {
        let hit = match segment.parse::<usize>() {
            Ok(nth) => nth_item(scope, nth),
            Err(_) => find_key(scope, segment),
        };
        let Some((line, next)) = hit else {
            break;
        };
        found = line;
        scope = next;
    }
    found
}

/// Direct child mapping key `key` of `scope`, with the value scope it owns.
fn find_key<'a, 'b>(scope: Scope<'a, 'b>, key: &str) -> Option<(usize, Scope<'a, 'b>)> {
    let region = scope.entries;
    let first = region.first()?;
    let item = scope.consumed > 0;
    // An unconsumed marker means the first entry opens a sequence, not a map.
    if item && first.marks.len() > scope.consumed {
        return None;
    }
    let level = match (item, first.text.is_empty()) {
        (true, false) => first.col,
        (true, true) => region.get(1)?.indent,
        (false, _) if first.dash() => return None,
        (false, _) => first.indent,
    };
    let at = region.iter().enumerate().position(|(index, entry)| {
        let direct = if item && index == 0 {
            entry.col == level
        } else {
            entry.indent == level && !entry.dash()
        };
        direct && key_matches(entry.text, key)
    })?;
    let owner = &region[at];
    let end = region[at + 1..]
        .iter()
        .position(|entry| {
            !(entry.indent > owner.col || (entry.indent == owner.col && entry.dash()))
        })
        .map_or(region.len(), |offset| at + 1 + offset);
    let next = Scope {
        entries: &region[at + 1..end],
        consumed: 0,
    };
    Some((owner.line, next))
}

/// The `nth` direct `- ` item of `scope`, including its own line.
fn nth_item<'a, 'b>(scope: Scope<'a, 'b>, nth: usize) -> Option<(usize, Scope<'a, 'b>)> {
    let region = scope.entries;
    let first = region.first()?;
    // Column of the sequence's markers. After a consumed marker the inner
    // sequence starts on the first entry itself (`- - k: 1`).
    let level = match scope.consumed {
        0 => first.dash().then_some(first.indent)?,
        done => *first.marks.get(done)?,
    };
    let is_item = |index: usize, entry: &Entry<'_>| {
        (index == 0 && scope.consumed > 0) || entry.marks.first() == Some(&level)
    };
    let at = region
        .iter()
        .enumerate()
        .filter(|(index, entry)| is_item(*index, entry))
        .map(|(index, _)| index)
        .nth(nth)?;
    let end = region[at + 1..]
        .iter()
        .position(|entry| entry.indent <= level)
        .map_or(region.len(), |offset| at + 1 + offset);
    let next = Scope {
        entries: &region[at..end],
        // Only the first entry still carries markers shared with its parent.
        consumed: if at == 0 { scope.consumed + 1 } else { 1 },
    };
    Some((region[at].line, next))
}
