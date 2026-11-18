use super::super::{
    verify,
    words::{self, Command, Word},
};

pub(super) struct UrlOperand<'a> {
    pub(super) at: usize,
    pub(super) word: &'a Word,
    pub(super) text: &'a str,
    pub(super) may_be_option: bool,
}

pub(super) struct OriginalShape<'a> {
    name: String,
    command_at: usize,
    argument_count: usize,
    url_words: Vec<(usize, &'a Word)>,
}

pub(super) fn original_shape(command: &Command) -> Option<OriginalShape<'_>> {
    let (name, command_at) = words::command_at(&command.words)?;
    let args = &command.words[command_at + 1..];
    let url_words = url_words(&name, args)
        .into_iter()
        .filter(|operand| operand.word.quoted_expansion.is_some())
        .map(|operand| (operand.at, operand.word))
        .collect::<Vec<_>>();
    if url_words.is_empty() {
        return None;
    }
    Some(OriginalShape {
        name,
        command_at,
        argument_count: args.len(),
        url_words,
    })
}

impl OriginalShape<'_> {
    pub(super) fn matches(&self, command: &str, at: usize, argument_count: usize) -> bool {
        self.name == command && self.command_at == at && self.argument_count == argument_count
    }

    pub(super) fn same_operand(&self, candidate: &UrlOperand<'_>) -> bool {
        self.url_words
            .binary_search_by_key(&candidate.at, |(at, _)| *at)
            .is_ok_and(|index| self.url_words[index].1 == candidate.word)
    }
}

pub(super) fn url_words<'a>(command: &str, args: &'a [Word]) -> Vec<UrlOperand<'a>> {
    let mut out = Vec::new();
    let mut index = 0;
    while let Some(word) = args.get(index) {
        let token = word.text.as_str();
        if token == "--" {
            out.extend(
                args.iter()
                    .enumerate()
                    .skip(index + 1)
                    .map(|(at, word)| UrlOperand {
                        at,
                        word,
                        text: &word.text,
                        may_be_option: false,
                    }),
            );
            break;
        }
        if let Some(text) = token.strip_prefix("--url=") {
            out.push(UrlOperand {
                at: index,
                word,
                text,
                may_be_option: false,
            });
        } else if verify::takes_url_slot(command, token) {
            index += 2;
            continue;
        } else if !token.starts_with('-') || token == "-" {
            out.push(UrlOperand {
                at: index,
                word,
                text: token,
                may_be_option: true,
            });
        }
        index += 1;
    }
    out
}
