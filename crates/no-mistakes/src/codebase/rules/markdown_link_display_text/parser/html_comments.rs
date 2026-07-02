pub(super) fn mask(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    let mut index = 0usize;
    while index + 3 < bytes.len() {
        if bytes[index..].starts_with(b"<!--") {
            let comment_start = index;
            index += 4;
            while index + 2 < bytes.len() && &bytes[index..index + 3] != b"-->" {
                index += 1;
            }
            let comment_end = if index + 2 < bytes.len() {
                index + 3
            } else {
                bytes.len()
            };
            for byte in &mut bytes[comment_start..comment_end] {
                if *byte != b'\n' {
                    *byte = b' ';
                }
            }
            index = comment_end;
        } else {
            index += 1;
        }
    }
    String::from_utf8(bytes).expect("comment masking preserves utf-8")
}
