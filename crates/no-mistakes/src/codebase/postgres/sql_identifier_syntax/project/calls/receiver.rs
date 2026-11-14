use super::*;

pub(super) fn classify(
    selection: &Selection,
    owner: Option<&ModuleBinding>,
    root: &Span,
    parts: &[String],
) -> (bool, bool) {
    let direct = parts.len() == 1 || (parts.len() == 2 && parts[1] == "query");
    let delayed = selection.delayed_receivers.contains(root) && direct;
    let typed = (delayed || owner.is_some_and(|binding| selection.typed.contains(&binding.id)))
        && (direct || input::indirect_executor_call(parts));
    (delayed, typed)
}
