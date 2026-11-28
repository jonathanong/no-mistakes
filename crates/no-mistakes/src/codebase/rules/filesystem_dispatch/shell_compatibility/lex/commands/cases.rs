use super::{command_words, Word};

#[derive(Default)]
pub(super) struct Cases(Vec<Phase>);

enum Phase {
    Subject,
    In,
    Pattern(usize),
    Body,
}

impl Cases {
    pub(super) fn word(&mut self, word: &str, words: &[Word<'_>]) -> bool {
        use Phase::{In, Pattern, Subject};
        match self.0.last_mut() {
            Some(phase @ Subject) => *phase = In,
            Some(phase @ In) if word == "in" => {
                *phase = Pattern(0);
                return true;
            }
            Some(Pattern(_)) => {
                if word == "esac" && words.is_empty() {
                    self.0.pop();
                }
            }
            _ if matches!(word, "case" | "esac") && command_words(words).next().is_none() => {
                if word == "case" {
                    self.0.push(Subject);
                } else {
                    self.0.pop();
                }
            }
            _ => {}
        }
        false
    }

    pub(super) fn in_pattern(&self) -> bool {
        matches!(self.0.last(), Some(Phase::Pattern(_)))
    }

    pub(super) fn punctuation(&mut self, byte: u8, first: bool) -> bool {
        match (self.0.last_mut(), byte) {
            (Some(Phase::Pattern(depth)), b'(') => *depth += usize::from(!first),
            (Some(phase @ Phase::Pattern(0)), b')') => {
                *phase = Phase::Body;
                return true;
            }
            (Some(Phase::Pattern(depth)), b')') => *depth -= 1,
            _ => {}
        }
        false
    }

    pub(super) fn next_arm(&mut self) {
        if let Some(phase @ Phase::Body) = self.0.last_mut() {
            *phase = Phase::Pattern(0);
        }
    }
}
