pub(super) fn after_parameter_list(header: &str) -> &str {
    let mut index = 0;
    let mut depth = 0i32;
    let mut saw_list = false;
    while index < header.len() {
        if let Some(end) = super::function_body::skip_noise(header, index) {
            index = end;
            continue;
        }
        let Some(character) = header[index..].chars().next() else {
            return header;
        };
        if character == '(' {
            depth += 1;
            saw_list = true;
        } else if character == ')' && saw_list {
            depth -= 1;
            if depth == 0 {
                return header.get(index + 1..).unwrap_or("");
            }
        }
        index += character.len_utf8();
    }
    header
}

pub(super) fn output_parameters(words: &[String]) -> String {
    let mut parts = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if words[index] == "out" || words[index] == "inout" {
            let mut chunk = vec![words[index].as_str()];
            index += 1;
            while index < words.len()
                && !matches!(
                    words[index].as_str(),
                    "in" | "out" | "inout" | "variadic" | ","
                )
                && !super::function_clauses::ends_return(&words[index])
            {
                chunk.push(words[index].as_str());
                index += 1;
            }
            if chunk.len() > 1 {
                parts.push(chunk.join(" "));
            }
            continue;
        }
        index += 1;
    }
    parts.join(" ")
}
