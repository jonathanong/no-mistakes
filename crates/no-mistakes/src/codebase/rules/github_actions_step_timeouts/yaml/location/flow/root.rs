use super::{flow_sequence_lines_from, is_content, value_after_anchor, FlowState};

fn mapping_key_len(text: &str, wanted: &str) -> Option<usize> {
    if text
        .strip_prefix(wanted)
        .is_some_and(|rest| rest.starts_with(':'))
    {
        return Some(wanted.len() + 1);
    }
    for quote in ['"', '\''] {
        if text.starts_with(quote)
            && text[quote.len_utf8()..]
                .strip_prefix(wanted)
                .is_some_and(|rest| {
                    rest.starts_with(quote) && rest[quote.len_utf8()..].starts_with(':')
                })
        {
            return Some(wanted.len() + 3);
        }
    }
    None
}

pub(crate) fn root_flow_job_steps(
    lines: &[&str],
    job_id: &str,
    count: usize,
) -> Option<Vec<usize>> {
    let mut state = FlowState::default();
    let mut ignored = Vec::new();
    let mut phase = 0;
    for (line_index, line) in lines.iter().enumerate() {
        let mut previous = None;
        for (column, character) in line.char_indices() {
            if state.quote.is_none() && character == '#' && previous.is_none_or(char::is_whitespace)
            {
                break;
            }
            let (depth, wanted) = match phase {
                0 => (1, "jobs"),
                1 => (2, job_id),
                _ => (3, "steps"),
            };
            if state.quote.is_none()
                && state.sequence_depth == 0
                && state.mapping_depth == depth
                && previous.is_none_or(|char| char.is_whitespace() || char == '{' || char == ',')
            {
                if let Some(length) = mapping_key_len(&line[column..], wanted) {
                    if phase == 2 {
                        let value = &line[column + length..];
                        let start = if value_after_anchor(value.trim_start()).starts_with('[') {
                            value
                                .find('[')
                                .map(|offset| (line_index, column + length + offset))
                        } else {
                            lines
                                .iter()
                                .enumerate()
                                .skip(line_index + 1)
                                .find(|(_, line)| is_content(line))
                                .and_then(|(index, line)| {
                                    value_after_anchor(line.trim_start())
                                        .starts_with('[')
                                        .then(|| (index, line.find('[').unwrap_or(0)))
                                })
                        };
                        return Some(start.map_or_else(
                            || vec![line_index + 1; count],
                            |(start, column)| {
                                flow_sequence_lines_from(
                                    lines,
                                    start,
                                    column,
                                    line_index + 1,
                                    count,
                                )
                            },
                        ));
                    }
                    phase += 1;
                }
            }
            state.visit(character, line_index + 1, &mut ignored);
            previous = Some(character);
        }
    }
    None
}
