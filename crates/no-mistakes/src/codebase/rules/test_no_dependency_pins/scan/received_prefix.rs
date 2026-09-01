use super::assertion_ranges::is_code;

pub(super) fn is_transparent(
    content: &str,
    assertion_start: usize,
    received_start: usize,
    non_code_ranges: &[(usize, usize)],
) -> bool {
    let mut prefix: String = content[assertion_start..received_start]
        .char_indices()
        .filter(|(offset, character)| {
            !character.is_whitespace() && is_code(non_code_ranges, assertion_start + offset)
        })
        .map(|(_, character)| character)
        .collect();
    trim_open_parens(&mut prefix);
    if prefix.ends_with("Promise.resolve") {
        prefix.truncate(prefix.len() - "Promise.resolve".len());
        trim_open_parens(&mut prefix);
    } else if let Some(callback) = ["async()=>", "()=>"]
        .into_iter()
        .find(|callback| prefix.ends_with(callback))
    {
        prefix.truncate(prefix.len() - callback.len());
        trim_open_parens(&mut prefix);
    }
    is_expect_base(&prefix)
}

fn trim_open_parens(prefix: &mut String) {
    while prefix.ends_with('(') {
        prefix.pop();
    }
}

fn is_expect_base(prefix: &str) -> bool {
    for base in ["expect", "expect!", "expect.soft", "expect.poll"] {
        if prefix == base
            || prefix
                .strip_prefix(base)
                .is_some_and(|arguments| arguments.starts_with('<') && arguments.ends_with('>'))
        {
            return true;
        }
    }
    false
}
