pub(super) fn start(bytes: &[u8], end: usize, non_code_ranges: &[(usize, usize)]) -> Option<usize> {
    if end == 0 || bytes[end - 1] != b'>' {
        return Some(end);
    }
    let mut depth = 0;
    let mut index = end;
    while index > 0 {
        index -= 1;
        let upper = non_code_ranges.partition_point(|(start, _)| *start <= index);
        if let Some(&(start, range_end)) = non_code_ranges[..upper].last() {
            if index < range_end {
                index = start;
                continue;
            }
        }
        match bytes[index] {
            b'>' if index == 0 || bytes[index - 1] != b'=' => depth += 1,
            b'<' if depth == 0 => return None,
            b'<' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}
