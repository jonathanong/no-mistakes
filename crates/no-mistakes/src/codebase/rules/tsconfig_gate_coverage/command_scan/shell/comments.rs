pub(super) fn contains_unquoted_hash(script: &str) -> bool {
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut escaped = false;
    for character in script.chars() {
        if escaped {
            escaped = false;
        } else if character == '\\' && !single_quoted {
            escaped = true;
        } else if character == '\'' && !double_quoted {
            single_quoted = !single_quoted;
        } else if character == '"' && !single_quoted {
            double_quoted = !double_quoted;
        } else if character == '#' && !single_quoted && !double_quoted {
            return true;
        }
    }
    false
}
