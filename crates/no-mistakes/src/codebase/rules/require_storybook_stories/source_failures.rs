use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) struct SourceFailures<'a> {
    facts: &'a CheckFactMap,
    failures: RefCell<BTreeMap<PathBuf, String>>,
    needed: RefCell<BTreeSet<PathBuf>>,
}

impl<'a> SourceFailures<'a> {
    pub(super) fn new(facts: &'a CheckFactMap) -> Self {
        Self {
            facts,
            failures: RefCell::default(),
            needed: RefCell::default(),
        }
    }

    pub(super) fn require(&self, path: &Path) {
        if !crate::codebase::dependencies::extract::is_indexable(path) {
            return;
        }
        self.needed.borrow_mut().insert(path.to_path_buf());
        let cause = match self.facts.ts.get(path) {
            None => Some(format!(
                "missing prepared Storybook source facts for {}",
                path.display()
            )),
            Some(facts) => facts
                .ts
                .operational_error
                .as_ref()
                .or(facts.parse_error.as_ref())
                .cloned()
                .or_else(|| {
                    facts
                        .ts
                        .fatal_parse_error
                        .then(|| format!("failed to parse {}", path.display()))
                }),
        };
        if let Some(cause) = cause {
            self.failures.borrow_mut().insert(path.to_path_buf(), cause);
        }
    }

    pub(super) fn require_story(&self, path: &Path) {
        self.require(path);
        if self
            .facts
            .ts
            .get(path)
            .is_some_and(|facts| facts.storybook.is_none())
        {
            self.missing(path, "Storybook");
        }
    }

    pub(super) fn require_symbols(&self, path: &Path) {
        if self
            .facts
            .ts
            .get(path)
            .is_some_and(|facts| facts.symbols.is_none())
        {
            self.missing(path, "import/export");
        }
    }

    pub(super) fn require_react(&self, path: &Path) {
        self.require(path);
        if self
            .facts
            .ts
            .get(path)
            .is_some_and(|facts| facts.react.is_none())
        {
            self.missing(path, "React component");
        }
    }

    fn missing(&self, path: &Path, kind: &str) {
        self.failures
            .borrow_mut()
            .entry(path.to_path_buf())
            .or_insert_with(|| format!("missing prepared {kind} facts for {}", path.display()));
    }

    pub(super) fn failed(&self) -> bool {
        !self.failures.borrow().is_empty()
    }
    pub(super) fn failed_file(&self, path: &Path) -> bool {
        self.failures.borrow().contains_key(path)
    }

    pub(super) fn finish(&self, session: Option<&AnalysisSession>) -> anyhow::Result<()> {
        if let Some(session) = session {
            session.register_check_demand(super::RULE_ID, self.needed.borrow().iter().cloned());
        }
        for (file, message) in self.failures.borrow().iter() {
            if let Some(session) = session.filter(|session| session.partial_checks_enabled()) {
                let mut failure = CheckFailure::new(super::RULE_ID, message);
                failure.file = Some(file.clone());
                session.record_check_failure(failure);
            } else {
                return Err(CheckInputFailure::source(file.clone(), message.clone()).into());
            }
        }
        Ok(())
    }
}
