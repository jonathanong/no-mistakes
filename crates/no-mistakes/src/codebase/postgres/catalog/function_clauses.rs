pub(super) struct HeaderModes {
    pub(super) null_input: String,
    pub(super) volatility: String,
    pub(super) security: String,
    pub(super) parallel: String,
    pub(super) return_contract: String,
}

pub(super) fn header_modes(definition: &str, span: Option<(usize, usize)>) -> HeaderModes {
    let words = words_outside_literals(&blank_body(definition, span));
    let mut null_input = "called";
    let mut volatility = "volatile";
    let mut security = "invoker";
    let mut parallel = "unsafe";
    for index in 0..words.len() {
        match words[index].as_str() {
            "strict" => null_input = "strict",
            "immutable" => volatility = "immutable",
            "stable" => volatility = "stable",
            "volatile" => volatility = "volatile",
            "called" if phrase_at(&words, index, &["called", "on", "null", "input"]) => {
                null_input = "called";
            }
            _ => {}
        }
        if phrase_at(&words, index, &["returns", "null", "on", "null", "input"]) {
            null_input = "strict";
        }
        if phrase_at(&words, index, &["security", "definer"]) {
            security = "definer";
        } else if phrase_at(&words, index, &["security", "invoker"]) {
            security = "invoker";
        }
        if phrase_at(&words, index, &["parallel", "restricted"]) {
            parallel = "restricted";
        } else if phrase_at(&words, index, &["parallel", "unsafe"]) {
            parallel = "unsafe";
        } else if phrase_at(&words, index, &["parallel", "safe"]) {
            parallel = "safe";
        }
    }
    HeaderModes {
        null_input: null_input.to_string(),
        volatility: volatility.to_string(),
        security: security.to_string(),
        parallel: parallel.to_string(),
        return_contract: return_contract(&words),
    }
}

fn return_contract(words: &[String]) -> String {
    let mut index = 0;
    while index < words.len() {
        if words[index] == "returns" {
            if phrase_at(words, index, &["returns", "null", "on", "null", "input"]) {
                index += 5;
                continue;
            }
            index += 1;
            let mut contract = Vec::new();
            while index < words.len() && !ends_return(&words[index]) {
                contract.push(words[index].as_str());
                index += 1;
            }
            return contract.join(" ");
        }
        index += 1;
    }
    String::new()
}

fn ends_return(word: &str) -> bool {
    matches!(
        word,
        "language"
            | "immutable"
            | "stable"
            | "volatile"
            | "leakproof"
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

fn blank_body(definition: &str, span: Option<(usize, usize)>) -> String {
    let Some((start, end)) = span else {
        return definition.to_string();
    };
    if start > end || end > definition.len() {
        return definition.to_string();
    }
    let mut text = String::with_capacity(definition.len());
    text.push_str(&definition[..start]);
    text.push(' ');
    text.push_str(&definition[end..]);
    text
}

fn words_outside_literals(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut index = 0;
    let mut current = String::new();
    while index < text.len() {
        if let Some(end) = super::function_body::skip_noise(text, index) {
            push_word(&mut words, &mut current);
            index = end;
            continue;
        }
        let Some(character) = text[index..].chars().next() else {
            break;
        };
        if character == '[' || character == ']' {
            push_word(&mut words, &mut current);
            if let Some(last) = words.last_mut() {
                last.push(character);
            } else {
                words.push(character.to_string());
            }
        } else if character.is_ascii_alphanumeric() || character == '_' {
            current.push(character.to_ascii_lowercase());
        } else {
            push_word(&mut words, &mut current);
        }
        index += character.len_utf8();
    }
    push_word(&mut words, &mut current);
    words
}

fn push_word(words: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

fn phrase_at(words: &[String], index: usize, phrase: &[&str]) -> bool {
    words[index..]
        .iter()
        .map(String::as_str)
        .take(phrase.len())
        .eq(phrase.iter().copied())
}
