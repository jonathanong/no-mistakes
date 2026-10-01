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

pub(super) fn after_set_clause(words: &[String], index: usize) -> usize {
    let mut cursor = index + 2;
    if matches!(words.get(cursor).map(String::as_str), Some("to" | "from")) {
        cursor += 1;
    }
    if cursor < words.len() {
        cursor += 1;
    }
    while cursor < words.len() {
        if words[cursor] == "," {
            cursor += 2;
            continue;
        }
        if option_boundary(&words[cursor]) {
            break;
        }
        cursor += 1;
    }
    cursor
}

fn option_boundary(word: &str) -> bool {
    matches!(
        word,
        "language"
            | "immutable"
            | "stable"
            | "volatile"
            | "leakproof"
            | "not"
            | "strict"
            | "called"
            | "security"
            | "cost"
            | "rows"
            | "support"
            | "parallel"
            | "transform"
            | "window"
            | "as"
            | "begin"
            | "set"
            | "returns"
    )
}

pub(super) fn contract_words(clauses: &[String], mut index: usize) -> Vec<String> {
    let mut contract = Vec::new();
    let mut depth = 0i32;
    while index < clauses.len() {
        let word = clauses[index].as_str();
        if word == "(" {
            depth += 1;
        } else if word == ")" {
            depth -= 1;
        } else if depth == 0 && ends_contract(clauses, index) {
            break;
        }
        contract.push(word.to_string());
        index += 1;
        if word == ")" && depth <= 0 {
            break;
        }
    }
    contract
}

fn ends_contract(clauses: &[String], index: usize) -> bool {
    super::function_clauses::ends_return(&clauses[index])
        || (clauses[index] == "not"
            && clauses
                .get(index + 1)
                .is_some_and(|word| word == "leakproof"))
}

pub(super) fn planner_clauses(clauses: &[String]) -> String {
    let mut parts = Vec::new();
    let mut index = 0;
    let mut depth = 0i32;
    while index < clauses.len() {
        if clauses[index] == "(" {
            depth += 1;
            index += 1;
            continue;
        }
        if clauses[index] == ")" {
            depth -= 1;
            index += 1;
            continue;
        }
        if clauses[index] == "set" {
            index = after_set_clause(clauses, index);
            continue;
        }
        if depth == 0 && clauses[index] == "support" {
            if let Some((clause, next)) = support_name(clauses, index) {
                parts.push(clause);
                index = next;
                continue;
            }
        }
        if depth == 0 && matches!(clauses[index].as_str(), "cost" | "rows") {
            if let Some(value) = clauses.get(index + 1) {
                if value != "(" && value != ")" {
                    parts.push(format!("{} {value}", clauses[index]));
                    index += 2;
                    continue;
                }
            }
        }
        index += 1;
    }
    parts.join(" ")
}

fn support_name(clauses: &[String], index: usize) -> Option<(String, usize)> {
    let first = clauses.get(index + 1)?;
    if first == "(" || first == ")" {
        return None;
    }
    let mut end = index + 2;
    if clauses.get(end).is_some_and(|word| name_continuation(word)) {
        end += 1;
    }
    Some((
        format!("support {}", clauses[index + 1..end].join(".")),
        end,
    ))
}

fn name_continuation(word: &str) -> bool {
    word != "(" && word != ")" && word != "," && !option_boundary(word)
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
