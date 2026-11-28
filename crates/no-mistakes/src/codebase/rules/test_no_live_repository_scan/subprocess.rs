//! Opt-in analyzer launches. The working directory is the scan root.
mod command;
mod place;

use super::config::Options;
use super::findings;
use super::RULE_ID;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::config::v2::schema::RuleDef;
use std::path::Path;

pub(super) struct Gate<'a> {
    root: &'a Path,
    application: &'a RuleDef,
    facts: &'a dyn TsFactLookup,
    options: &'a Options,
    fixture_dirs: Vec<&'a str>,
}

impl<'a> Gate<'a> {
    pub(super) fn new(
        root: &'a Path,
        application: &'a RuleDef,
        facts: &'a dyn TsFactLookup,
        options: &'a Options,
        fixture_dirs: &[&'a str],
    ) -> Self {
        Self {
            root,
            application,
            facts,
            options,
            fixture_dirs: fixture_dirs.to_vec(),
        }
    }

    pub(super) fn note(&self, diagnostics: &mut AnalysisDiagnostics, site: &ResolvedCallSite) {
        if self.options.analyzers.is_empty() {
            return;
        }
        let Some(name) = command::launcher_name(site) else {
            return;
        };
        let Some(facts) = self.facts.get_ts_facts(&site.file) else {
            return;
        };
        let Some(arguments) =
            super::super::call_policy::call_argument_values(facts, site.offset)
        else {
            return;
        };
        let shell = command::uses_shell(name, &arguments.values);
        let finding = |message: String, target: &str| {
            findings::build(self.root, self.application, site, message, target)
        };
        for invocation in command::launches(&arguments.values, shell) {
            let Some(label) = command::matched(&invocation, &self.options.analyzers) else {
                continue;
            };
            let wrapped = command::wrapper_dir(&invocation);
            let base = place::directory(facts, site.offset, invocation.options, &self.fixture_dirs);
            let mut live_dirs = Vec::new();
            let mut allowed = false;
            let mut unresolved = false;
            for shell_dir in invocation.shell_dirs.as_deref().unwrap_or(&[None]) {
                let dir = wrapped.as_deref().or(shell_dir.as_deref());
                let located = dir.map_or(base, |dir| {
                    place::shell_place(dir, &self.fixture_dirs, base)
                });
                match located {
                    place::Directory::Live => live_dirs.push(dir),
                    place::Directory::Allowed => allowed = true,
                    place::Directory::Unresolved => unresolved = true,
                }
            }
            if !live_dirs.is_empty() {
                diagnostics.findings.push(finding(
                    place::live_message(
                        &site.source_callee,
                        &label,
                        place::owned_shift(base, wrapped.as_deref(), &live_dirs),
                        allowed || unresolved,
                    ),
                    "analyzer subprocess",
                ));
                return;
            }
            if unresolved {
                findings::unresolved_findings(diagnostics, self.options.unresolved_root).push(
                    finding(
                        format!(
                            "unresolved root: `{}` runs `{label}`, but its cwd cannot be shown to be a fixtures path or an owned temporary directory. Fix: pass a const fixture path or owned mkdtemp directory as `cwd`. As a last option, add `no-mistakes-disable-next-line {RULE_ID}` with a reason.",
                            site.source_callee
                        ),
                        "unresolved root",
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "subprocess/gate_tests.rs"]
mod gate_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "subprocess/place_support.rs"]
mod place_support;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "subprocess/place_tests.rs"]
mod place_tests;
