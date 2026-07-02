pub(super) fn is_inside_string(bytes: &[u8], target: usize) -> bool {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        String(u8),
        Template,
        TemplateExpression { depth: usize },
    }

    let mut stack = vec![State::Normal];
    let mut index = 0usize;
    while index < target {
        let state = *stack.last().unwrap_or(&State::Normal);
        match (state, bytes[index]) {
            (State::Normal, b'\'' | b'"') => {
                stack.push(State::String(bytes[index]));
                index += 1;
            }
            (State::Normal, b'`') => {
                stack.push(State::Template);
                index += 1;
            }
            (State::String(_), b'\\') | (State::Template, b'\\') => index += 2,
            (State::String(active), byte) if byte == active => {
                stack.pop();
                index += 1;
            }
            (State::Template, b'`') => {
                stack.pop();
                index += 1;
            }
            (State::Template, b'$') if bytes.get(index + 1) == Some(&b'{') => {
                stack.push(State::TemplateExpression { depth: 1 });
                index += 2;
            }
            (State::TemplateExpression { .. }, b'\'' | b'"') => {
                stack.push(State::String(bytes[index]));
                index += 1;
            }
            (State::TemplateExpression { .. }, b'`') => {
                stack.push(State::Template);
                index += 1;
            }
            (State::TemplateExpression { depth }, b'{') => {
                *stack.last_mut().expect("template expression state exists") =
                    State::TemplateExpression { depth: depth + 1 };
                index += 1;
            }
            (State::TemplateExpression { depth: 1 }, b'}') => {
                stack.pop();
                index += 1;
            }
            (State::TemplateExpression { depth }, b'}') => {
                *stack.last_mut().expect("template expression state exists") =
                    State::TemplateExpression { depth: depth - 1 };
                index += 1;
            }
            _ => index += 1,
        }
    }
    matches!(stack.last(), Some(State::String(_)) | Some(State::Template))
}
