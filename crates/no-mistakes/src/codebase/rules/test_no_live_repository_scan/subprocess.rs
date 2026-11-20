//! Opt-in analyzer launches. The working directory is the scan root.
mod command;

use super::config::Options;
use super::findings;
use super::provenance::{classify, Verdict};
use super::RULE_ID;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;
use crate::config::v2::schema::RuleDef;
use command::OptionsSlot;
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
            super::super::forbidden_calls::call_argument_values(facts, site.offset)
        else {
            return;
        };
        let shell = command::uses_shell(name, &arguments.values);
        let Some(invocation) = command::invocation(&arguments.values, shell) else {
            return;
        };
        let Some(label) = command::matched(&invocation, &self.options.analyzers) else {
            return;
        };
        let finding = |message: String, target: &str| {
            findings::build(self.root, self.application, site, message, target)
        };
        let place = match invocation.shell_dir.as_deref() {
            Some(dir) => shell_place(dir, &self.fixture_dirs),
            None => directory(facts, site.offset, invocation.options, &self.fixture_dirs),
        };
        match place {
            Directory::Allowed => {}
            Directory::Live => diagnostics.findings.push(finding(
                format!(
                    "live repository scan: `{}` runs `{label}` against the working directory, and a test root reaches it. `analyzers` names that command, and this call passes no owned `cwd`. Fix: run it in static CI, or set `cwd` to a fixtures path or an mkdtemp directory the test owns.",
                    site.source_callee
                ),
                "analyzer subprocess",
            )),
            Directory::Unresolved => {
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

enum Directory {
    Allowed,
    Live,
    Unresolved,
}

fn shell_place(dir: &str, fixture_dirs: &[&str]) -> Directory {
    let segments: Vec<&str> = dir
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();
    if segments.is_empty() || segments.contains(&"..") {
        return Directory::Unresolved;
    }
    if segments
        .iter()
        .any(|segment| fixture_dirs.contains(segment))
    {
        Directory::Allowed
    } else {
        Directory::Live
    }
}

fn directory(
    facts: &TsFileFacts,
    offset: u32,
    options: OptionsSlot,
    fixture_dirs: &[&str],
) -> Directory {
    let OptionsSlot::Object(index) = options else {
        return match options {
            OptionsSlot::Unknown => Directory::Unresolved,
            _ => Directory::Live,
        };
    };
    match cwd_origin(facts, offset, index) {
        Some(origin) => match classify(Some(origin), fixture_dirs) {
            Verdict::Allowed => Directory::Allowed,
            Verdict::Live(_) => Directory::Live,
            Verdict::Unresolved(_) => Directory::Unresolved,
        },
        None if object_closed(facts, offset, index) => Directory::Live,
        None => Directory::Unresolved,
    }
}

fn object_closed(facts: &TsFileFacts, offset: u32, index: usize) -> bool {
    facts
        .module_bindings
        .as_deref()
        .and_then(|bindings| bindings.path_provenance.as_deref())
        .and_then(|provenance| provenance.arg_closed_objects.get(&offset))
        .is_some_and(|indexes| indexes.contains(&index))
}

fn cwd_origin(facts: &TsFileFacts, offset: u32, index: usize) -> Option<&PathOrigin> {
    facts
        .module_bindings
        .as_deref()?
        .path_provenance
        .as_deref()?
        .arg_member_paths
        .get(&offset)?
        .iter()
        .find_map(|(member_index, key, origin)| {
            (*member_index == index && key == "cwd").then_some(origin)
        })
}

#[cfg(test)]
#[path = "subprocess/gate_tests.rs"]
mod gate_tests;
