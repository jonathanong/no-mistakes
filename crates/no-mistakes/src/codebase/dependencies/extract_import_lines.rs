fn import_line_starts(source: &str) -> Vec<u32> {
    if source.is_empty() {
        return Vec::new();
    }
    let mut starts = vec![0u32];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push((index + 1) as u32);
        }
    }
    starts
}


fn import_line_at(line_starts: &[u32], byte_offset: usize) -> u32 {
    if line_starts.is_empty() {
        return 1;
    }
    let offset = byte_offset as u32;
    match line_starts.binary_search(&offset) {
        Ok(index) => index as u32 + 1,
        Err(index) => index as u32,
    }
}
