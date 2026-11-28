use super::super::super::lex::Word;
use crate::fx::FxHashSet;

const EXTERNAL_TEST_OPS: &[&str] = &[
    "-b", "-c", "-d", "-e", "-f", "-g", "-h", "-k", "-o", "-p", "-r", "-s", "-t", "-u", "-w", "-x",
    "-G", "-L", "-N", "-O", "-S", "-ef", "-nt", "-ot",
];

pub(super) fn is_uncorrelatable_predicate(words: &[&Word<'_>]) -> bool {
    has_external_test_predicate(words)
        || words.iter().any(|w| has_untracked_dollar(w.raw))
        || !is_modeled_condition(words)
}

fn is_modeled_condition(words: &[&Word<'_>]) -> bool {
    let body = if words.first().is_some_and(|word| word.raw == "!") {
        &words[1..]
    } else {
        words
    };
    body.first()
        .is_some_and(|head| matches!(head.raw, "[[" | "[" | "test") || head.raw.starts_with("(("))
}

pub(super) fn has_volatile_var(vars: &FxHashSet<String>) -> bool {
    vars.iter().any(|v| {
        matches!(
            v.as_str(),
            "RANDOM"
                | "SECONDS"
                | "LINENO"
                | "SRANDOM"
                | "EPOCHSECONDS"
                | "EPOCHREALTIME"
                | "BASHPID"
        )
    })
}

fn has_external_test_predicate(words: &[&Word<'_>]) -> bool {
    let body = if words.first().is_some_and(|w| w.raw == "!") {
        &words[1..]
    } else {
        words
    };
    body.split_first().is_some_and(|(head, rest)| {
        matches!(head.raw, "[[" | "[" | "test")
            && rest
                .iter()
                .any(|w| EXTERNAL_TEST_OPS.contains(&w.raw) || w.raw == "-a")
    })
}

fn has_untracked_dollar(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let (mut i, mut double_quoted) = (0, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\'' if !double_quoted => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                i += 1;
            }
            b'"' => {
                double_quoted = !double_quoted;
                i += 1;
            }
            b'$' => {
                i += 1;
                let braced = bytes.get(i) == Some(&b'{');
                if braced {
                    i += 1;
                }
                if is_special_param(bytes, i, braced) {
                    return true;
                }
            }
            _ => i += 1,
        }
    }
    false
}

fn is_special_param(bytes: &[u8], i: usize, braced: bool) -> bool {
    match bytes.get(i) {
        Some(b'?' | b'@' | b'*' | b'$' | b'-' | b'0'..=b'9') => true,
        Some(b'#') => {
            !braced
                || !bytes
                    .get(i + 1)
                    .is_some_and(|&b| b == b'_' || b.is_ascii_alphabetic())
        }
        // ${!name} reads whichever variable `name` holds. That target is not tracked.
        Some(b'!') => {
            !braced
                || bytes
                    .get(i + 1)
                    .is_some_and(|&b| b == b'_' || b.is_ascii_alphabetic())
        }
        _ => false,
    }
}
