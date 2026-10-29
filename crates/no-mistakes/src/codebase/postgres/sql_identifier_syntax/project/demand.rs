use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;

impl Syntax<'_> {
    pub(super) fn finish(
        &mut self,
        module: &TypeScriptModuleFacts,
        selections: &[Selection],
        output: &mut FileFacts,
        program: &Program<'_>,
        source: &str,
    ) {
        if self.recover_helpers {
            if retain(module, selections, output, &self.recovery) {
                self.recovery.validate(&self.template_binds);
                self.recovery.iterators =
                    helper_recovery::iterator::collect(program, module, source);
            } else {
                // No selected call, configured root, or runtime export can reach this model.
                self.recovery = Default::default();
            }
        }
        output.recovery = std::mem::take(&mut self.recovery);
    }
}

fn retain(
    module: &TypeScriptModuleFacts,
    selections: &[Selection],
    output: &FileFacts,
    recovery: &helper_recovery::FileModels,
) -> bool {
    output.profiles.iter().any(|(_, calls)| !calls.is_empty())
        || selections.iter().any(Selection::retains_recovery)
        || module.exports.iter().any(|export| !export.type_only)
        || !module.diagnostics.is_empty()
        || recovery
            .executors
            .values()
            .any(|bindings| !bindings.is_empty())
        || recovery.sinks.values().any(|calls| !calls.is_empty())
}
