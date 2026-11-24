use super::fact_reads::specifier_at_offset;
use super::load_lex::LexState;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, FxHashMap};

pub(super) fn matching_parens(source: &str) -> FxHashMap<usize, usize> {
    let mut pairs = fx_map();
    let mut stack = Vec::new();
    let mut lex = LexState::new();
    let mut chars = source.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        match lex.step(source, &mut chars, index, ch) {
            Some(true) => continue,
            Some(false) => {}
            None => break,
        }
        if ch == '(' {
            stack.push(index);
        } else if ch == ')' {
            if let Some(open) = stack.pop() {
                pairs.insert(open, index);
            }
        }
    }
    pairs
}

pub(super) fn depth_zero_callee<'a>(
    ts: &TsFileFacts,
    window: &'a str,
    window_start: u32,
) -> Option<(usize, &'a str)> {
    depth_zero_callee_with_jumps(ts, window, window_start, None)
}

pub(super) fn depth_zero_callee_with_jumps<'a>(
    ts: &TsFileFacts,
    window: &'a str,
    window_start: u32,
    jumps: Option<&FxHashMap<usize, usize>>,
) -> Option<(usize, &'a str)> {
    let spans = &ts.namespaces.literal_import_spans;
    let first = spans.partition_point(|(offset, _)| *offset < window_start);
    let mut recorded = spans[first..].iter().peekable();
    let mut depth = 0i32;
    let mut base = 0usize;
    let mut lex = LexState::new();
    let mut chars = window.char_indices().peekable();
    while let Some((rel, ch)) = chars.next() {
        let index = base + rel;
        if lex.step(&window[base..], &mut chars, rel, ch)? {
            continue;
        }
        if ch == '(' {
            if depth == 0 {
                let abs = window_start as usize + index;
                if let Some(&close_abs) = jumps.and_then(|map| map.get(&abs)) {
                    let close_rel = close_abs.saturating_sub(window_start as usize);
                    if close_abs > abs && close_rel < window.len() {
                        base = close_rel + 1;
                        chars = window[base..].char_indices().peekable();
                        lex.reset_after_jump();
                        continue;
                    }
                }
            }
            depth += 1;
            continue;
        }
        if ch == ')' && depth > 0 {
            depth -= 1;
            continue;
        }
        if depth != 0 {
            continue;
        }
        let rest = &window[index..];
        let offset = window_start as u64 + index as u64;
        while recorded
            .peek()
            .is_some_and(|(start, _)| u64::from(*start) < offset)
        {
            recorded.next();
        }
        let has_fact = recorded
            .peek()
            .is_some_and(|(start, _)| u64::from(*start) == offset);
        if !has_fact && !callee_at_start(rest, "import") && !callee_at_start(rest, "require") {
            continue;
        }
        let prev = window[..index].chars().next_back();
        if prev.is_some_and(|prev| {
            prev == '.' || prev == '_' || prev == '$' || prev.is_ascii_alphanumeric()
        }) {
            continue;
        }
        return Some((index, rest));
    }
    None
}

pub(super) fn window_loads_module(ts: &TsFileFacts, window: &str, start: u32) -> bool {
    let rhs = window.split_once('=').map(|(_, rhs)| rhs).unwrap_or(window);
    let call = rhs.trim_start();
    let call = call
        .strip_prefix("await")
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .map(str::trim_start)
        .unwrap_or(call);
    let offset = start.saturating_add(u32::try_from(window.len() - call.len()).unwrap_or(u32::MAX));
    let recorded = specifier_at_offset(ts, offset).is_some();
    direct_module_load(call.trim_end().trim_end_matches(';'), recorded)
}

pub(super) fn direct_module_load(text: &str, recorded: bool) -> bool {
    let rest = if recorded {
        text.find('(').map(|index| &text[index..])
    } else {
        text.strip_prefix("import")
            .or_else(|| text.strip_prefix("require"))
    };
    let Some(rest) = rest.and_then(|rest| rest.trim_start().strip_prefix('(')) else {
        return false;
    };
    close_call(rest).is_some_and(|after| after.trim().is_empty())
}

fn close_call(text: &str) -> Option<&str> {
    let mut depth = 1i32;
    let mut lex = LexState::new();
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if lex.step(text, &mut chars, index, ch)? {
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return text.get(index + ch.len_utf8()..);
                }
            }
            _ => {}
        }
    }
    None
}

fn callee_at_start(text: &str, callee: &str) -> bool {
    text.strip_prefix(callee)
        .is_some_and(|rest| rest.trim_start().starts_with('('))
}
