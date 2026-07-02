pub(super) fn mask(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut active: Option<&'static str> = None;
    for line in source.split_inclusive('\n') {
        if let Some(tag) = active {
            push_masked(line, &mut out);
            if contains_close_tag(line, tag) {
                active = None;
            }
            continue;
        }

        if let Some(tag) = opening_raw_block_tag(line) {
            push_masked(line, &mut out);
            if !contains_close_tag(line, tag) {
                active = Some(tag);
            }
        } else {
            out.push_str(line);
        }
    }
    out
}

fn opening_raw_block_tag(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    ["pre", "script", "style"]
        .into_iter()
        .find(|tag| starts_with_open_tag(trimmed, tag))
}

fn starts_with_open_tag(value: &str, tag: &str) -> bool {
    let Some(rest) = value.strip_prefix('<') else {
        return false;
    };
    let rest = rest.strip_prefix(tag).or_else(|| {
        rest.get(..tag.len())
            .filter(|prefix| prefix.eq_ignore_ascii_case(tag))
            .and_then(|_| rest.get(tag.len()..))
    });
    rest.is_some_and(|rest| {
        rest.starts_with('>')
            || rest.starts_with(' ')
            || rest.starts_with('\t')
            || rest.starts_with('\n')
    })
}

fn contains_close_tag(line: &str, tag: &str) -> bool {
    line.to_ascii_lowercase()
        .contains(&format!("</{}>", tag.to_ascii_lowercase()))
}

fn push_masked(line: &str, out: &mut String) {
    out.extend(line.chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }));
}
