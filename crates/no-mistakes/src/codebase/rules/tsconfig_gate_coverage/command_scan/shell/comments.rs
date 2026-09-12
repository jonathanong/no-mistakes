pub(super) fn strip_static_comments(script: &str) -> String {
    let mut output = String::with_capacity(script.len());
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut escaped = false;
    let mut comment = false;
    for character in script.chars() {
        if comment {
            if character == '\n' {
                comment = false;
                output.push(character);
            }
            continue;
        }
        if escaped {
            escaped = false;
        } else if character == '\\' && !single_quoted {
            escaped = true;
        } else if character == '\'' && !double_quoted {
            single_quoted = !single_quoted;
        } else if character == '"' && !single_quoted {
            double_quoted = !double_quoted;
        } else if character == '#'
            && !single_quoted
            && !double_quoted
            && output
                .chars()
                .next_back()
                .is_none_or(|previous| previous.is_whitespace() || ";|&()<>".contains(previous))
        {
            comment = true;
            continue;
        }
        output.push(character);
    }
    output
}
