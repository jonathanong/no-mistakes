use super::{command_words, identifier, Word};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CompoundBody {
    If,
    Case,
    Loop,
    Bracket,
}

impl CompoundBody {
    pub(super) fn from_opener(raw: &str) -> Option<Self> {
        match raw {
            "if" => Some(Self::If),
            "case" => Some(Self::Case),
            "for" | "while" | "until" | "select" => Some(Self::Loop),
            "[[" => Some(Self::Bracket),
            _ => None,
        }
    }

    pub(super) fn closer(self) -> &'static str {
        match self {
            Self::If => "fi",
            Self::Case => "esac",
            Self::Loop => "done",
            Self::Bracket => "]]",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Group {
    Brace,
    CoprocBrace,
    Subshell,
    Function(bool),
    Compound {
        body: CompoundBody,
        depth: usize,
        is_function: bool,
    },
}

pub(super) fn is_coproc_prefix(words: &[Word<'_>]) -> bool {
    let mut cmd = command_words(words);
    if cmd.next().is_none_or(|word| word.raw != "coproc") {
        return false;
    }
    match (cmd.next(), cmd.next()) {
        (None, _) => true,
        (Some(name), None) => identifier(name.raw),
        _ => false,
    }
}

pub(super) fn at_keyword_position(words: &[Word<'_>]) -> bool {
    command_words(words).all(|word| matches!(word.raw, "!" | "then" | "else" | "elif" | "do"))
}

pub(super) fn function_keyword_named<'a>(words: &[Word<'a>]) -> Option<Function<'a>> {
    let mut words = command_words(words);
    if words.next()?.raw != "function" {
        return None;
    }
    let name = words.next()?;
    words.next().is_none().then_some(Function::Named(name.raw))
}

pub(super) fn closes(group: Option<&Group>, byte: u8) -> bool {
    matches!(
        (group, byte),
        (
            Some(Group::Brace | Group::CoprocBrace | Group::Function(true)),
            b'}'
        ) | (Some(Group::Subshell | Group::Function(false)), b')')
    )
}

pub(super) fn group_start(words: &[Word<'_>], is_bash: bool) -> bool {
    command_words(words)
        .next()
        .is_none_or(|word| word.raw == "function")
        || (is_bash && is_coproc_prefix(words))
}

#[derive(Clone, Copy)]
pub(super) enum Function<'a> {
    Named(&'a str),
    Anonymous,
}

impl<'a> Function<'a> {
    pub(super) fn name(self) -> Option<&'a str> {
        match self {
            Self::Named(name) => Some(name),
            Self::Anonymous => None,
        }
    }
}

pub(super) fn function_keyword<'a>(words: &[Word<'a>]) -> Option<Function<'a>> {
    let mut words = command_words(words);
    if words.next()?.raw != "function" {
        return None;
    }
    Some(
        words
            .next()
            .map_or(Function::Anonymous, |word| Function::Named(word.raw)),
    )
}

pub(super) fn function_parentheses<'a>(words: &[Word<'a>], is_zsh: bool) -> Option<Function<'a>> {
    let mut command = command_words(words);
    let Some(word) = command.next() else {
        return is_zsh.then_some(Function::Anonymous);
    };
    if word.raw == "function" {
        function_keyword(words)
    } else {
        let valid_name = is_zsh || identifier(word.raw);
        (valid_name && command.next().is_none()).then_some(Function::Named(word.raw))
    }
}
