pub(super) fn starts_with(source: &[u8], i: usize, needle: &[u8]) -> bool {
    source.get(i..).is_some_and(|rest| rest.starts_with(needle))
}

pub(super) fn repeated(source: &[u8], i: usize, byte: u8, width: usize) -> bool {
    source
        .get(i..i + width)
        .is_some_and(|range| range.iter().all(|&candidate| candidate == byte))
}

pub(super) fn blank_range(masked: &mut [u8], start: usize, end: usize) {
    for i in start..end {
        blank(masked, i);
    }
}

pub(super) fn blank(masked: &mut [u8], i: usize) {
    if !matches!(masked[i], b'\n' | b'\r') {
        masked[i] = b' ';
    }
}

pub(super) fn utf8_width(first: u8) -> usize {
    match first {
        0..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}
