use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValue, ArrayGuard};
use crate::fx::FxHashSet;

impl Owners<'_> {
    pub(super) fn prune_infeasible_callers(&mut self, facts: &dyn TsFactLookup) {
        let excluded: FxHashSet<_> = self
            .callers
            .values()
            .flatten()
            .filter(|site| self.infeasible(site, facts))
            .map(|site| (site.file.clone(), site.offset))
            .collect();
        for callers in self.callers.values_mut() {
            callers.retain(|site| !excluded.contains(&(site.file.clone(), site.offset)));
        }
    }

    fn infeasible(&self, site: &ResolvedCallSite, facts: &dyn TsFactLookup) -> bool {
        let Some(guards) = facts
            .get_ts_facts(&site.file)
            .and_then(path_provenance)
            .and_then(|paths| paths.call_guards.get(&site.offset))
        else {
            return false;
        };
        if self.gate_roots.contains(&site.file) {
            return false;
        }
        let source = site_source_node(site);
        if !self.proven_target(&source) || self.closure_unknown(&site.file) {
            return false;
        }
        let Some(callers) = self.callers.get(&source).filter(|sites| !sites.is_empty()) else {
            return false;
        };
        callers.iter().all(|caller| {
            let Some(arguments) = facts
                .get_ts_facts(&caller.file)
                .filter(|facts| path_provenance(facts).is_some())
                .and_then(|file| file.module_bindings.as_ref())
                .and_then(|module| module.call_arguments.as_ref())
                .and_then(|calls| calls.get(&caller.offset))
            else {
                return false;
            };
            arguments.spread_from.is_none()
                && guards.iter().any(|guard| {
                    arguments
                        .values
                        .get(guard.parameter)
                        .is_some_and(|value| contradicts(guard, value))
                })
        })
    }
}

fn contradicts(guard: &ArrayGuard, value: &ArgumentValue) -> bool {
    let ArgumentValue::Array(values) = value else {
        return false;
    };
    if guard.length {
        return guard.expected.as_f64() != Some(values.len().saturating_sub(guard.index) as f64);
    }
    match values.get(guard.index) {
        Some(ArgumentValue::Scalar(value)) => {
            if value.is_number() && guard.expected.is_number() {
                value.as_f64() != guard.expected.as_f64()
            } else {
                value != &guard.expected
            }
        }
        None => true,
        _ => false,
    }
}
