pub(super) fn is_code_line(bytes: &[u8], index: usize) -> bool {
    let line_start = bytes[..index]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |pos| pos + 1);
    columns(std::str::from_utf8(&bytes[line_start..index]).unwrap_or("")) > 3
}

pub(super) fn columns(line: &str) -> usize {
    let mut columns = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => columns += 1,
            '\t' => columns += 4 - (columns % 4),
            _ => break,
        }
    }
    columns
}
