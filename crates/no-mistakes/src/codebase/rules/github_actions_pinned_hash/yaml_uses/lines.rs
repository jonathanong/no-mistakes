use super::indent::{in_input_block, is_ignorable, is_nested_key, is_value_line};

/// Job and step `uses` lines, in document order. A root sequence counts as steps.
/// Lines nested under `with` or `env` do not.
pub(super) fn action_uses_lines(lines: &[&str]) -> Vec<usize> {
    let mut found = Vec::new();
    // `{ uses:` with the ref on the next line stays open across blanks.
    let mut await_value = false;
    for index in 0..lines.len() {
        if in_input_block(lines, index) {
            await_value = false;
            continue;
        }
        let value_here = is_value_line(lines[index]) && !is_nested_key(lines[index]);
        // `uses:` then `nested: true` is a mapping, not the action scalar.
        let continued = index > 0
            && bare_uses_key(lines[index - 1])
            && !in_input_block(lines, index - 1)
            && value_here;
        let kind = classify_uses_line(lines[index]);
        if kind == LineUses::Present || continued || (await_value && value_here) {
            found.push(index + 1);
        }
        if !is_ignorable(lines[index]) {
            await_value = kind == LineUses::Awaits;
        }
    }
    found
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LineUses {
    None,
    Present,
    Awaits,
}

fn bare_uses_key(line: &str) -> bool {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('-').map(str::trim).unwrap_or(trimmed);
    matches!(trimmed, "uses:" | "\"uses\":" | "'uses':")
}

/// `Present` is a uses scalar on this line. `Awaits` is `{ uses:` whose ref
/// starts on a later line.
fn classify_uses_line(line: &str) -> LineUses {
    let mut quote: Option<char> = None;
    let mut token = String::new();
    let mut scalar = String::new();
    let mut after_colon = false;
    let mut key_was_uses = false;
    let mut flow_depth = 0_i32;
    let mut flow_input: Vec<bool> = Vec::new();
    // Set at `with:` / `env:` and consumed by the following `{` or `[`.
    let mut value_key_is_input = false;
    let mut skip_anchor = false;
    let mut alias = false;

    for ch in line.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
                if after_colon && finish_scalar(&scalar, key_was_uses) {
                    return LineUses::Present;
                }
                if after_colon {
                    scalar.clear();
                    key_was_uses = false;
                    after_colon = false;
                }
            } else if after_colon {
                scalar.push(ch);
            } else {
                token.push(ch);
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            if !after_colon {
                token.clear();
            }
            continue;
        }
        if ch == '#' {
            break;
        }
        // `&anchor value` and `*alias` sit between `uses:` and the scalar.
        // An alias is the whole scalar; a delimiter must not stay inside the name.
        if skip_anchor {
            let boundary = ch.is_whitespace() || matches!(ch, '}' | ']' | ',' | ':');
            if !boundary {
                continue;
            }
            if alias && key_was_uses {
                return LineUses::Present;
            }
            skip_anchor = false;
            alias = false;
            if ch.is_whitespace() {
                continue;
            }
        }
        if after_colon && scalar.is_empty() && matches!(ch, '&' | '*') {
            alias = ch == '*';
            skip_anchor = true;
            continue;
        }
        if matches!(ch, '{' | '[') {
            let entering = flow_input.last().copied().unwrap_or(false) || value_key_is_input;
            flow_input.push(entering);
            value_key_is_input = false;
            flow_depth += 1;
            token.clear();
            scalar.clear();
            after_colon = false;
            key_was_uses = false;
            continue;
        }
        if matches!(ch, '}' | ']' | ',') {
            if alias && key_was_uses {
                return LineUses::Present;
            }
            if finish_scalar(&scalar, key_was_uses) {
                return LineUses::Present;
            }
            if ch != ',' {
                flow_depth -= 1;
                flow_input.pop();
            }
            value_key_is_input = false;
            token.clear();
            scalar.clear();
            after_colon = false;
            key_was_uses = false;
            continue;
        }
        if ch.is_whitespace() || (ch == '-' && !after_colon && flow_depth == 0 && token.is_empty())
        {
            if after_colon && !scalar.is_empty() {
                if finish_scalar(&scalar, key_was_uses) {
                    return LineUses::Present;
                }
                // A block value ends the line. Inside `{...}` another key may follow.
                if flow_depth == 0 {
                    return LineUses::None;
                }
                scalar.clear();
                key_was_uses = false;
                after_colon = false;
            } else if !after_colon {
                token.clear();
            }
            continue;
        }
        if ch == ':' && !after_colon {
            let in_input = flow_input.last().copied().unwrap_or(false);
            value_key_is_input = !in_input && (token == "with" || token == "env");
            key_was_uses = token == "uses" && !in_input;
            token.clear();
            scalar.clear();
            after_colon = true;
            continue;
        }
        if after_colon {
            value_key_is_input = false;
            scalar.push(ch);
        } else {
            token.push(ch);
        }
    }
    if (alias && key_was_uses) || finish_scalar(&scalar, key_was_uses) {
        return LineUses::Present;
    }
    // `{ uses:` — the flow collection is still open and the ref is later.
    if key_was_uses && scalar.is_empty() && flow_depth > 0 {
        return LineUses::Awaits;
    }
    LineUses::None
}

fn finish_scalar(scalar: &str, key_was_uses: bool) -> bool {
    key_was_uses && !scalar.is_empty()
}
