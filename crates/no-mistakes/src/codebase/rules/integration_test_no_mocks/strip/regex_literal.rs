pub(super) fn push_erased(bytes: &[u8], start: usize, out: &mut String) -> usize {
    out.push(' ');
    let mut index = start + 1;
    let mut in_class = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => super::push_erased_escape(bytes, &mut index, out),
            b'\n' => {
                out.push('\n');
                return index + 1;
            }
            b'[' => {
                in_class = true;
                out.push(' ');
                index += 1;
            }
            b']' if in_class => {
                in_class = false;
                out.push(' ');
                index += 1;
            }
            b'/' if !in_class => {
                out.push(' ');
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
                    out.push(' ');
                    index += 1;
                }
                return index;
            }
            _ => {
                out.push(' ');
                index += 1;
            }
        }
    }
    index
}

pub(super) fn can_start(out: &str) -> bool {
    let trimmed = out.trim_end();
    let Some(previous) = trimmed.as_bytes().last().copied() else {
        return true;
    };
    matches!(
        previous,
        b'(' | b'['
            | b'{'
            | b','
            | b';'
            | b':'
            | b'='
            | b'!'
            | b'?'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'~'
            | b'^'
            | b'<'
            | b'>'
    ) || trimmed.ends_with("return")
        || trimmed.ends_with("throw")
        || trimmed.ends_with("case")
}
