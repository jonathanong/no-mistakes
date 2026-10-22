use super::CheckFileFacts;
use std::sync::Arc;

pub(super) fn into_shared(mut file: CheckFileFacts) -> Arc<CheckFileFacts> {
    // Relative/imported executor and annotation projection have finished. Later
    // checks borrow only the projected call prefixes, not the evaluator program.
    // Retain each profile and its calls while releasing its owned program state.
    for (_, annotation) in &mut file.query_annotation {
        let calls = std::mem::take(&mut annotation.calls);
        *annotation = Default::default();
        annotation.calls = calls;
    }
    Arc::new(file)
}
