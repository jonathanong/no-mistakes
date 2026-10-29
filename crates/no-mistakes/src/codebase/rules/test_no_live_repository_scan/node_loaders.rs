//! Conservative fallback for potential Node-origin loaders in prepared facts.
use super::{config::Enumerator, requires::is_enumerator_module, RuleDef};
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn findings(
    root: &Path,
    application: &RuleDef,
    site: &ResolvedCallSite,
    facts: &dyn TsFactLookup,
    extra: &[Enumerator],
) -> Vec<RuleFinding> {
    facts.get_ts_facts(&site.file)
        .and_then(|facts| facts.module_bindings.as_ref())
        .and_then(|module| module.node_loader_calls.get(&site.offset))
        .into_iter().flatten()
        .filter(|load| load.computed || is_enumerator_module(&load.specifier, extra))
        .map(|load| {
            let argument = if load.computed { load.specifier.clone() }
                else { format!("{:?}", load.specifier) };
            let expression = format!("{}({argument})", load.callee);
            RuleFinding {
                rule: super::RULE_ID.into(),
                file: relative_slash_path(root, &site.file),
                line: site.line as usize,
                message: application.message.clone().unwrap_or_else(|| format!(
                    "unresolved root: `{expression}` may load a directory enumerator through a Node module loader. Its uses and roots cannot be shown to be bounded. Fix: use a statically named import and enumerate a path under fixtures or an owned mkdtemp directory, or add `no-mistakes-disable-next-line {}` with a reason.", super::RULE_ID
                )),
                import: Some(expression),
                target: Some("unresolved root".into()),
            }
        }).collect()
}
