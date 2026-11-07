use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::NodeId;

impl Proof<'_, '_> {
    pub(super) fn call(
        &mut self,
        file: &Path,
        span: (u32, u32),
        frame: Option<&Frame>,
        awaited: bool,
    ) -> bool {
        let key = Key::Call(file.to_path_buf(), span.0, span.1, awaited, frame.cloned());
        self.memoized(key, |proof| proof.returned(file, span, frame, awaited))
    }

    fn returned(
        &mut self,
        file: &Path,
        span: (u32, u32),
        frame: Option<&Frame>,
        awaited: bool,
    ) -> bool {
        let Some(call) = self
            .prepared(file)
            .and_then(|data| data.calls.get(&span))
            .copied()
        else {
            return false;
        };
        if call.bound || call.spread_at.is_some() {
            return false;
        }
        let Some((producer, owner)) = target(self.graph, file, span.0) else {
            return false;
        };
        let Some(ts) = self.facts.get_ts_facts(&producer) else {
            return false;
        };
        if ts.value_flow.generator_callables.contains(&owner)
            || (ts.value_flow.async_callables.contains(&owner) && !awaited)
        {
            return false;
        }
        let Some(returned) = self
            .prepared(&producer)
            .and_then(|data| data.returns.get(&owner))
            .cloned()
        else {
            return false;
        };
        let generated = self
            .prepared(&producer)
            .and_then(|data| data.generated.get(&owner))
            .cloned();
        let arguments = call
            .arguments
            .iter()
            .map(|value| {
                (call.arguments.len() == 1 && call.last_argument_literal)
                    || self.expression(file, value, frame, false)
            })
            .collect();
        let next = Frame {
            file: producer.clone(),
            owner,
            arguments,
        };
        if let Some(prefixes) = generated.filter(|prefixes| prefixes.len() == returned.len()) {
            !prefixes.is_empty()
                && prefixes
                    .iter()
                    .all(|prefix| self.expression(&producer, prefix, Some(&next), false))
        } else {
            !returned.is_empty()
                && returned
                    .iter()
                    .all(|value| self.expression(&producer, value, Some(&next), awaited))
        }
    }
}

fn target(graph: &DepGraph, file: &Path, offset: u32) -> Option<(PathBuf, CallableId)> {
    let sites = graph.call_sites_in_file(file);
    let start = sites.partition_point(|site| site.offset < offset);
    let mut sites = sites[start..]
        .iter()
        .take_while(|site| site.offset == offset)
        .filter(|site| site.invocation == InvocationKind::Call);
    let site = sites.next()?;
    if sites.next().is_some()
        || !site.alternatives.is_empty()
        || graph.failed_call_sources(site).next().is_some()
    {
        return None;
    }
    let NodeId::Symbol {
        file,
        callable_id: Some(owner),
        ..
    } = site.target_node.as_ref()?
    else {
        return None;
    };
    Some((file.as_ref().to_path_buf(), *owner))
}
