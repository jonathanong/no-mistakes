pub fn heredoc_at(line: &str) -> Option<usize> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut arith = 0i32;
    let mut index = 0;
    while index < chars.len() {
        if starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if starts(&chars, index, &['<', '<', '<']) {
            index += 3;
            continue;
        }
        if arith == 0 && starts(&chars, index, &['<', '<']) {
            return Some(chars[index].0);
        }
        index += if starts(&chars, index, &['<', '<']) {
            2
        } else {
            1
        };
    }
    None
}

fn starts(chars: &[(usize, char)], index: usize, prefix: &[char]) -> bool {
    prefix.iter().enumerate().all(|(offset, expected)| {
        chars
            .get(index + offset)
            .is_some_and(|(_, ch)| ch == expected)
    })
}
