use super::{indent, is_content};
use crate::codebase::rules::github_actions_step_timeouts::yaml::{
    value_after_anchor, yaml_key_value,
};

fn flow_start(lines: &[&str], key: (usize, usize)) -> Option<usize> {
    if yaml_key_value(lines[key.0])
        .is_some_and(|(_, value)| value_after_anchor(value).starts_with('['))
    {
        return Some(key.0);
    }
    lines
        .iter()
        .enumerate()
        .skip(key.0 + 1)
        .find(|(_, line)| is_content(line))
        .and_then(|(index, line)| {
            (indent(line) > key.1 && value_after_anchor(line.trim_start()).starts_with('['))
                .then_some(index)
        })
}

#[derive(Default)]
struct FlowState {
    sequence_depth: usize,
    mapping_depth: usize,
    quote: Option<char>,
    escaped: bool,
    awaiting_item: bool,
}

impl FlowState {
    fn visit(&mut self, character: char, line: usize, result: &mut Vec<usize>) -> bool {
        if let Some(quote) = self.quote {
            if self.escaped {
                self.escaped = false;
            } else if quote == '"' && character == '\\' {
                self.escaped = true;
            } else if character == quote {
                self.quote = None;
            }
            return false;
        }
        if character == '"' || character == '\'' {
            self.quote = Some(character);
        }
        if self.sequence_depth == 1
            && self.mapping_depth == 0
            && self.awaiting_item
            && !character.is_whitespace()
            && character != ']'
        {
            result.push(line);
            self.awaiting_item = false;
        }
        match character {
            '[' => {
                self.sequence_depth += 1;
                if self.sequence_depth == 1 {
                    self.awaiting_item = true;
                }
            }
            ']' => {
                self.sequence_depth -= 1;
                if self.sequence_depth == 0 {
                    return true;
                }
            }
            '{' => self.mapping_depth += 1,
            '}' => self.mapping_depth -= 1,
            ',' if self.sequence_depth == 1 && self.mapping_depth == 0 => {
                self.awaiting_item = true;
            }
            _ => {}
        }
        false
    }
}

pub(super) fn flow_sequence_item_lines(
    lines: &[&str],
    key: (usize, usize),
    count: usize,
) -> Vec<usize> {
    let Some(start) = flow_start(lines, key) else {
        return vec![key.0 + 1; count];
    };
    let mut result = Vec::with_capacity(count);
    let mut state = FlowState::default();
    for (index, line) in lines.iter().enumerate().skip(start) {
        let text = if index == start {
            &line[line.find('[').unwrap_or(0)..]
        } else {
            line
        };
        let mut previous = None;
        for character in text.chars() {
            if state.quote.is_none() && character == '#' && previous.is_none_or(char::is_whitespace)
            {
                break;
            }
            if state.visit(character, index + 1, &mut result) {
                if start > key.0 && index == start {
                    return vec![key.0 + 1; count];
                }
                result.truncate(count);
                result.resize(count, key.0 + 1);
                return result;
            }
            previous = Some(character);
        }
    }
    result.truncate(count);
    result.resize(count, key.0 + 1);
    result
}
