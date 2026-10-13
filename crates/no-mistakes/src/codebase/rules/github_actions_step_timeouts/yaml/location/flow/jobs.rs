use super::{indent, is_content, value_after_anchor, yaml_key_value, FlowState};

pub(crate) fn jobs_flow_start(lines: &[&str], jobs: (usize, usize)) -> Option<(usize, usize)> {
    let (_, value) = yaml_key_value(lines[jobs.0])?;
    let start = if value_after_anchor(value).starts_with('{') {
        lines[jobs.0].find('{').map(|column| (jobs.0, column))
    } else if value.is_empty() {
        lines
            .iter()
            .enumerate()
            .skip(jobs.0 + 1)
            .find(|(_, line)| is_content(line))
            .and_then(|(index, line)| {
                (indent(line) > jobs.1 && value_after_anchor(line.trim_start()).starts_with('{'))
                    .then(|| (index, line.find('{').unwrap_or(0)))
            })
    } else {
        None
    }?;
    (!closes_on_line(&lines[start.0][start.1..])).then_some(start)
}

fn closes_on_line(text: &str) -> bool {
    let mut state = FlowState::default();
    let mut ignored = Vec::new();
    let mut previous = None;
    let mut opened = false;
    for character in text.chars() {
        if state.quote.is_none() && character == '#' && previous.is_none_or(char::is_whitespace) {
            break;
        }
        state.visit(character, 1, &mut ignored);
        opened |= state.mapping_depth > 0;
        if opened && state.mapping_depth == 0 {
            return true;
        }
        previous = Some(character);
    }
    false
}
