//! A helper call's bounded view of its module and prepared imports.

use super::Shared;
use crate::codebase::postgres::{
    run_helper_program, Callee, Callees, EmbeddedSqlOptions, HelperProgram, SiteArg, MAX_CALL_DEPTH,
};
use std::path::{Path, PathBuf};

/// One module's view while its helper runs.
pub(super) struct Frame<'s, 'a, 'o, R> {
    pub(super) shared: &'s mut Shared<'a, R>,
    pub(super) profile: usize,
    pub(super) options: &'o EmbeddedSqlOptions,
    /// The module whose executor is being completed.
    pub(super) root: PathBuf,
    pub(super) file: PathBuf,
    pub(super) depth: u8,
    pub(super) stack: Vec<(PathBuf, String)>,
}

impl<'s, 'a, 'o, R> Frame<'s, 'a, 'o, R> {
    pub(super) fn new(
        shared: &'s mut Shared<'a, R>,
        profile: usize,
        options: &'o EmbeddedSqlOptions,
        from: &Path,
    ) -> Self {
        Self {
            shared,
            profile,
            options,
            root: from.to_path_buf(),
            file: from.to_path_buf(),
            depth: 0,
            stack: Vec::new(),
        }
    }
}

impl<R: FnMut(&str, &Path) -> Option<PathBuf>> Callees for Frame<'_, '_, '_, R> {
    fn call(&mut self, name: &str, args: &[SiteArg]) -> Callee {
        if self.depth >= MAX_CALL_DEPTH {
            return Callee::Failed;
        }
        let Some((origin, local)) = self.shared.find(&self.file, name, None) else {
            return Callee::Unresolved;
        };
        if self
            .stack
            .iter()
            .any(|(file, known)| *file == origin && *known == local)
        {
            return Callee::Failed;
        }
        let key = (
            self.profile,
            self.root.clone(),
            origin.clone(),
            local.clone(),
            args.to_vec(),
        );
        if let Some(known) = self.shared.cache.get(&key) {
            return known.clone().map_or(Callee::Failed, Callee::Done);
        }
        let Some(program) = helper_program(self.shared, self.options, &origin, &local) else {
            return Callee::Unresolved;
        };
        let mut stack = self.stack.clone();
        stack.push((origin.clone(), local));
        let mut child = Frame {
            shared: &mut *self.shared,
            profile: self.profile,
            options: self.options,
            root: self.root.clone(),
            file: origin.clone(),
            depth: self.depth + 1,
            stack,
        };
        let stamp = (origin != self.root).then_some(origin.as_path());
        let outcomes = run_helper_program(&program, args, stamp, &mut child);
        self.shared.cache.insert(key, outcomes.clone());
        outcomes.map_or(Callee::Failed, Callee::Done)
    }
}

pub(super) fn helper_program<R>(
    shared: &Shared<'_, R>,
    options: &EmbeddedSqlOptions,
    origin: &Path,
    local: &str,
) -> Option<std::sync::Arc<HelperProgram>> {
    let (_, embedded) = shared
        .files
        .get(origin)?
        .embedded_sql
        .iter()
        .find(|(known, _)| known == options)?;
    embedded.exported_programs.get(local).cloned()
}
