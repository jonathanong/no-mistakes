pub(super) struct Word {
    pub(super) text: String,
    pub(super) line: usize,
}

#[derive(Default)]
pub(super) struct Command {
    pub(super) words: Vec<Word>,
    pub(super) dynamic: bool,
}

mod tokenize;

pub(super) fn commands(script: &str) -> Vec<Command> {
    tokenize::commands(script)
}
