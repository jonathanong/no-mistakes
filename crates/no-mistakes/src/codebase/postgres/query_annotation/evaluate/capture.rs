use super::Environment;
use crate::fx::FxHashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

/// A lexical frame uses a checked one-based handle. Vec-backed scope indices
/// are bounded below usize::MAX; zero remains a compiler enum niche rather
/// than an application-defined module-frame sentinel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct LexicalFrame(NonZeroUsize);
impl From<Environment> for LexicalFrame {
    fn from(env: Environment) -> Self {
        Self(
            env.checked_add(1)
                .and_then(NonZeroUsize::new)
                .expect("lexical frame index fits a nonzero handle"),
        )
    }
}
impl LexicalFrame {
    pub(super) fn index(self) -> Environment {
        self.0.get() - 1
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) enum Capture {
    Lexical(LexicalFrame),
    Module,
}
impl From<Environment> for Capture {
    fn from(env: Environment) -> Self {
        Self::Lexical(env.into())
    }
}
impl Capture {
    // Resolve only values live in the current evaluator world. Terminal events
    // can outlive speculative resets, but their SQL projection never resolves
    // or invokes function captures.
    pub(super) fn environment(
        self,
        owner: &Path,
        modules: &FxHashMap<PathBuf, Environment>,
    ) -> Environment {
        match self {
            Self::Lexical(env) => env.index(),
            Self::Module => *modules
                .get(owner)
                .expect("module capture retains its registered frame"),
        }
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> super::Evaluator<'_, F> {
    pub(super) fn capture(&self, owner: &Path, env: Environment) -> Capture {
        if self.modules.get(owner) == Some(&env) {
            Capture::Module
        } else {
            env.into()
        }
    }
}
