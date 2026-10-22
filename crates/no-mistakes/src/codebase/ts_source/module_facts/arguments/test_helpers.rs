use super::{collect_selected, ArgumentValues};
use crate::fx::FxHashMap;
use oxc_semantic::Semantic;

pub(crate) fn collect(
    semantic: &Semantic<'_>,
    facts: &super::super::TypeScriptModuleFacts,
) -> FxHashMap<u32, ArgumentValues> {
    collect_selected(semantic, facts, None)
}
