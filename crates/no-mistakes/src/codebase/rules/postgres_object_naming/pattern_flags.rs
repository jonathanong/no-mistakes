pub(super) struct FlagSpan {
    pub(super) end: usize,
    pub(super) scoped: bool,
}

pub(super) fn flag_span(chars: &[(usize, char)], start: usize) -> Option<FlagSpan> {
    if chars
        .get(start + 1)
        .is_none_or(|(_, character)| *character != '?')
    {
        return None;
    }
    let mut index = start + 2;
    let mut dash = false;
    let mut flag = false;
    while let Some((_, character)) = chars.get(index) {
        match character {
            '-' if !dash => dash = true,
            'i' | 'm' | 's' | 'u' | 'U' | 'x' | 'R' => flag = true,
            ')' if flag => {
                return Some(FlagSpan {
                    end: index,
                    scoped: false,
                });
            }
            ':' if flag => {
                return Some(FlagSpan {
                    end: index,
                    scoped: true,
                });
            }
            _ => return None,
        }
        index += 1;
    }
    None
}

pub(super) fn apply_verbose(body: &str, verbose: &mut bool) {
    let (on, off) = body.split_once('-').unwrap_or((body, ""));
    if on.contains('x') {
        *verbose = true;
    }
    if off.contains('x') {
        *verbose = false;
    }
}
