pub(super) fn is_atx_heading(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let hashes = line[indent..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    (1..=6).contains(&hashes)
        && line
            .get(indent + hashes)
            .is_none_or(|byte| matches!(byte, b' ' | b'\t'))
}

pub(super) fn is_thematic_break(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let Some(&marker @ (b'*' | b'-' | b'_')) = line.get(indent) else {
        return false;
    };
    let mut markers = 0;
    for byte in &line[indent..] {
        if *byte == marker {
            markers += 1;
        } else if !matches!(byte, b' ' | b'\t') {
            return false;
        }
    }
    markers >= 3
}

pub(super) fn is_setext_heading_underline(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let Some(&marker @ (b'=' | b'-')) = line.get(indent) else {
        return false;
    };
    let markers = line[indent..]
        .iter()
        .take_while(|byte| **byte == marker)
        .count();
    line[indent + markers..]
        .iter()
        .all(|byte| matches!(byte, b' ' | b'\t'))
}

pub(super) fn starts_block_container(line: &[u8], in_paragraph: bool) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let Some(marker) = line.get(indent) else {
        return false;
    };
    if *marker == b'>' {
        return true;
    }
    if marker.is_ascii_digit() {
        return ordered_list_starts_block(&line[indent..], in_paragraph);
    }
    if !matches!(marker, b'*' | b'+' | b'-') {
        return false;
    }
    let remainder = &line[indent + 1..];
    (remainder.is_empty() || matches!(remainder.first(), Some(b' ' | b'\t')))
        && (!in_paragraph || has_nonblank(remainder))
}

fn ordered_list_starts_block(line: &[u8], in_paragraph: bool) -> bool {
    let digits = line.iter().take_while(|byte| byte.is_ascii_digit()).count();
    if !(1..=9).contains(&digits) || !matches!(line.get(digits), Some(b'.' | b')')) {
        return false;
    }
    let remainder = &line[digits + 1..];
    if !remainder.is_empty() && !matches!(remainder.first(), Some(b' ' | b'\t')) {
        return false;
    }
    let value = line[..digits]
        .iter()
        .fold(0_u64, |value, digit| value * 10 + u64::from(digit - b'0'));
    !in_paragraph || (value == 1 && has_nonblank(remainder))
}

fn has_nonblank(line: &[u8]) -> bool {
    line.iter().any(|byte| !matches!(byte, b' ' | b'\t'))
}

pub(super) fn is_indented_code(line: &[u8]) -> bool {
    let mut columns = 0;
    for byte in line {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns += 4 - (columns % 4),
            _ => return columns >= 4,
        }
    }
    false
}
