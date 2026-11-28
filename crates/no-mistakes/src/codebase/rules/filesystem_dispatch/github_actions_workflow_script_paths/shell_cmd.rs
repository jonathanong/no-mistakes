use crate::codebase::rules::github_actions_download_integrity::words::Word;

const CLUSTERED_C_FLAGS: &str = "abcefhkmnptuvxBCEHPTilrs";

pub(super) fn shell_c_operand(words: &[Word], mut index: usize) -> Option<&Word> {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if text == "--command" || takes_clustered_c(text) {
            return words.get(index + 1);
        }
        if !text.starts_with('-') {
            return None;
        }
        index += 1 + usize::from(matches!(text, "-o" | "-O" | "--rcfile"));
    }
    None
}

fn takes_clustered_c(text: &str) -> bool {
    let Some(cluster) = text.strip_prefix('-').filter(|body| !body.starts_with('-')) else {
        return false;
    };
    let Some((before, after)) = cluster.split_once('c') else {
        return false;
    };
    before
        .chars()
        .chain(after.chars())
        .all(|ch| CLUSTERED_C_FLAGS.contains(ch))
}
