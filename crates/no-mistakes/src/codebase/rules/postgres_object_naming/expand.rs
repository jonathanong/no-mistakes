pub(super) fn middle_matches(
    middle: &str,
    table: &str,
    abbreviations: bool,
    min_letters: usize,
) -> bool {
    middle.eq_ignore_ascii_case(table) || (abbreviations && expands(middle, table, min_letters))
}

pub(super) fn expands(middle: &str, table: &str, min_letters: usize) -> bool {
    let abbreviated = split_keep(middle);
    let words = split_keep(table);
    abbreviated.len() == words.len()
        && abbreviated
            .iter()
            .zip(words)
            .all(|(abbrev, word)| part_matches(abbrev, word, min_letters))
}

pub(super) fn suggestion(table: &str, min_letters: usize) -> Option<String> {
    let suggested = split_keep(table)
        .iter()
        .map(|part| suggest_part(part, min_letters))
        .collect::<Vec<_>>()
        .join("_");
    (suggested != table).then_some(suggested)
}

fn part_matches(abbrev: &str, word: &str, min_letters: usize) -> bool {
    if abbrev.is_empty() || word.is_empty() {
        return abbrev.is_empty() && word.is_empty();
    }
    let abbreviated: Vec<char> = abbrev.chars().collect();
    let word: Vec<char> = word.chars().collect();
    abbreviated[0].eq_ignore_ascii_case(&word[0])
        && is_subsequence(&abbreviated[1..], &word[1..])
        && abbreviated.len() >= min_letters.min(word.len())
}

fn is_subsequence(needle: &[char], haystack: &[char]) -> bool {
    let mut rest = haystack;
    for character in needle {
        let Some(found) = rest
            .iter()
            .position(|candidate| candidate.eq_ignore_ascii_case(character))
        else {
            return false;
        };
        rest = &rest[found + 1..];
    }
    true
}

fn suggest_part(word: &str, min_letters: usize) -> String {
    if word.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = word.chars().collect();
    let mut kept = String::new();
    kept.push(chars[0]);
    for character in chars.iter().skip(1) {
        if !is_vowel(*character) {
            kept.push(*character);
        }
    }
    let need = min_letters.min(chars.len());
    if kept.chars().count() < need {
        chars.iter().take(need).collect()
    } else {
        kept
    }
}

fn is_vowel(character: char) -> bool {
    matches!(character.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
}

fn split_keep(value: &str) -> Vec<&str> {
    value.split('_').collect()
}
