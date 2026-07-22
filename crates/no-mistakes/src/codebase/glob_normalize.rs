pub(crate) fn normalize(pattern: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in pattern.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                let mut literal_parent = false;
                if let Some(parent) = parts.last() {
                    literal_parent = true;
                    for ch in parent.chars() {
                        if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
                            literal_parent = false;
                            break;
                        }
                    }
                }
                if literal_parent {
                    parts.pop();
                } else {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}
