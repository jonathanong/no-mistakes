use super::hot_files;
use no_mistakes_ts_facts::hooks::{DiagnosticsHooks, FilePass};
use std::path::Path;

struct NoMistakesTsFactsHooks;

fn map_pass(pass: FilePass) -> hot_files::FilePass {
    match pass {
        FilePass::Parse => hot_files::FilePass::Parse,
        FilePass::ValueFlowExtract => hot_files::FilePass::ValueFlowExtract,
        FilePass::ValueFlowFinalize => hot_files::FilePass::ValueFlowFinalize,
    }
}

impl DiagnosticsHooks for NoMistakesTsFactsHooks {
    fn record_work(&self, metric: &'static str, count: u64) {
        if let Some(observer) = super::current() {
            observer.increment(metric, count);
        }
    }

    fn trace_file(&self, pass: FilePass, path: &Path, units: u64, operation: &mut dyn FnMut()) {
        hot_files::trace(map_pass(pass), path, units, operation);
    }

    fn trace_current_file(&self, pass: FilePass, units: u64, operation: &mut dyn FnMut()) {
        hot_files::trace_current(map_pass(pass), units, operation);
    }
}

static TS_FACTS_HOOKS: NoMistakesTsFactsHooks = NoMistakesTsFactsHooks;

pub(crate) fn ensure_registered() {
    no_mistakes_ts_facts::hooks::register_diagnostics_hooks(&TS_FACTS_HOOKS);
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
