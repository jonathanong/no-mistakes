use super::super::{verify, words::Word};
use super::{dynamic, operands::UrlOperand, url};

pub(super) fn unproven_argument<'a>(
    command: &str,
    args: &'a [Word],
    candidates: &[UrlOperand<'a>],
    indirect_command: bool,
) -> Option<(&'a Word, &'a str)> {
    // An expansion can become an option and consume a later literal URL.
    dynamic_option(command, args)
        .or_else(|| {
            candidates
                .iter()
                .find(|operand| dynamic(operand.text))
                .map(|operand| (operand.word, operand.text))
        })
        .or_else(|| {
            if indirect_command {
                candidates
                    .iter()
                    .find(|operand| url::http_url(operand.text).is_some())
                    .map(|operand| (operand.word, operand.text))
            } else {
                None
            }
        })
}

/// Dynamic option names can change whether a later token is a URL or an operand.
pub(super) fn dynamic_option<'a>(command: &str, args: &'a [Word]) -> Option<(&'a Word, &'a str)> {
    let mut index = 0;
    while let Some(word) = args.get(index) {
        let token = word.text.as_str();
        if token == "--" {
            break;
        }
        if verify::takes_url_slot(command, token) {
            if let Some(operand) = args.get(index + 1).filter(|word| word.expansion_may_split) {
                return Some((operand, operand.text.as_str()));
            }
            index += 2;
            continue;
        }
        let dynamic_name = if token.starts_with("--") {
            dynamic(token.split_once('=').map_or(token, |(name, _)| name))
                || (token.contains('=') && word.expansion_may_split)
        } else if token.starts_with('-') {
            dynamic_short_name(command, word)
        } else {
            false
        };
        if dynamic_name {
            return Some((word, token));
        }
        index += 1;
    }
    None
}

fn dynamic_short_name(command: &str, word: &Word) -> bool {
    for character in word.text.chars().skip(1) {
        if matches!(character, '$' | '`' | '{') {
            return true;
        }
        if character.is_ascii() {
            let flag = [b'-', character as u8];
            if verify::takes_url_slot(command, std::str::from_utf8(&flag).unwrap_or("")) {
                // Quoting -o"$OUT" keeps its value in this word; unquoted
                // expansion can add another option even with an attached value.
                return word.expansion_may_split;
            }
        }
    }
    false
}
