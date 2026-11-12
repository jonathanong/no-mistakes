mod entries;
mod selection;
use entries::diagnostic_paths;
mod text;
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::ts_source::module_facts::arguments::{EnvDiagnostic, PathOrigin};
use std::path::PathBuf;

#[derive(Clone)]
pub(super) struct Entry {
    file: PathBuf,
    offset: Option<u32>,
    line: u32,
    concrete: bool,
}

pub(in super::super) struct EnvCause {
    pub(super) hint: EnvDiagnostic,
    pub(super) file: PathBuf,
    pub(super) line: u32,
    pub(super) reason: &'static str,
    pub(super) concrete: bool,
}

impl Owners<'_> {
    pub(super) fn env_cause(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> Option<EnvCause> {
        let hint = self.environment_hint(origin, site, facts)?;
        let source = site_source_node(site);
        let module_scope = matches!(&source, NodeId::File(_));
        let mut entries = self
            .entries(source)
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| !self.env_argument_owned(&hint.name, None, entry, facts))
            .map(|entry| Entry {
                file: entry.file.clone(),
                offset: Some(entry.offset),
                line: entry.line,
                concrete: true,
            })
            .collect::<Vec<_>>();
        if entries.is_empty() || module_scope {
            entries = self.environment_entries(&site.file, facts);
        }
        if entries.is_empty() {
            entries.push(Entry {
                file: site.file.clone(),
                offset: None,
                line: site.line,
                concrete: false,
            });
        }
        let reason = if self.env_owned(&hint.name, None, site, facts) {
            if hint.captured {
                "does not prove the captured environment read was initialized with an owned root"
            } else {
                "reaches a derived path that is not proven to stay inside the owned environment root"
            }
        } else {
            "has no proven assignment of a fixture or owned mkdtemp root before reaching this scan"
        };
        entries
            .into_iter()
            .map(|entry| self.rank_environment_entry(entry, hint.clone(), reason, facts))
            .min_by(|left, right| {
                (&left.0, &left.1.file, left.2).cmp(&(&right.0, &right.1.file, right.2))
            })
            .map(|(_, cause, _)| cause)
    }

    fn environment_hint(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> Option<EnvDiagnostic> {
        if let Some(hint) = &origin.env_diagnostic {
            return Some((**hint).clone());
        }
        let offset = origin.getter.or(origin.captured_getter)?;
        let call = self
            .graph
            .call_sites_in_file(&site.file)
            .iter()
            .find(|call| call.offset == offset)?;
        let mut hint = getter_environment_hint(call, facts)?;
        hint.captured |= origin.captured_getter.is_some();
        Some(hint)
    }
}

pub(super) fn getter_environment_hint(
    call: &ResolvedCallSite,
    facts: &dyn TsFactLookup,
) -> Option<EnvDiagnostic> {
    if !call.alternatives.is_empty() {
        return None;
    }
    let NodeId::Symbol {
        file,
        callable_id: Some(id),
        ..
    } = call.target_node.as_ref()?
    else {
        return None;
    };
    let hint = facts
        .get_ts_facts(file)
        .and_then(diagnostic_paths)?
        .env_getters
        .get(&id.0)?
        .env_diagnostic
        .as_deref()?
        .clone();
    Some(hint)
}

#[cfg(test)]
mod tests;
