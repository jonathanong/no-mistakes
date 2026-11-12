use super::super::env;
use super::diagnostic_paths;
use super::{Entry, EnvCause, EnvDiagnostic, Owners};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::module_facts::arguments::WriteScope;

impl Owners<'_> {
    pub(super) fn rank_environment_entry(
        &self,
        entry: Entry,
        hint: EnvDiagnostic,
        reason: &'static str,
        facts: &dyn TsFactLookup,
    ) -> (u8, EnvCause, Option<u32>) {
        let mut cause = EnvCause {
            hint,
            file: entry.file.clone(),
            line: entry.line,
            reason,
            concrete: entry.concrete,
        };
        if !entry.concrete {
            return (6, cause, entry.offset);
        }
        let Some(offset) = entry.offset else {
            return (4, cause, entry.offset);
        };
        let Some(paths) = facts.get_ts_facts(&entry.file).and_then(diagnostic_paths) else {
            return (4, cause, entry.offset);
        };
        let name = &cause.hint.name;
        let hook = paths.env_writes.iter().any(|write| {
            write.name == *name
                && env::owned_origin(&write.origin, &self.fixture_dirs)
                && match write.scope {
                    WriteScope::Hook(start, end) | WriteScope::BeforeEach(start, end) => {
                        paths.test_callbacks.iter().any(|(from, to)| {
                            start <= *from && *to <= end && *from <= offset && offset < *to
                        })
                    }
                    _ => false,
                }
        });
        let setup = hook
            || self
                .imported
                .owns(&entry.file, name, facts, &self.consumed, &self.fixture_dirs)
            || self.setups.owns(
                &entry.file,
                name,
                facts,
                &self.imported,
                &self.consumed,
                &self.fixture_dirs,
            );
        let write = paths
            .env_writes
            .iter()
            .filter(|write| {
                write.name == *name
                    && write.end <= offset
                    && match write.scope {
                        WriteScope::Program => !setup,
                        WriteScope::Function(start, end) => start <= offset && offset < end,
                        _ => false,
                    }
            })
            .max_by_key(|write| write.end);
        if let Some(write) = write {
            let local = matches!(write.scope, WriteScope::Function(..));
            if write.cleared {
                cause.line = write.line;
                cause.reason = "clears that environment variable before reaching this scan";
                return (if local { 0 } else { 2 }, cause, entry.offset);
            }
            if !env::owned_origin(&write.origin, &self.fixture_dirs) {
                cause.line = write.line;
                cause.reason = "assigns an environment value that is not a proven fixture or owned mkdtemp root";
                return (if local { 1 } else { 3 }, cause, entry.offset);
            }
            cause.reason =
                "does not prove the captured or derived environment read used that owned root";
            return (5, cause, entry.offset);
        }
        if setup {
            cause.reason = "does not prove the captured or derived environment read used the configured owned root";
            return (5, cause, entry.offset);
        }
        (4, cause, entry.offset)
    }
}
