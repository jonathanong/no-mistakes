//! Advisory-only member selection never grants executor ownership.
use super::*;

pub(super) fn unproved_receiver(
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    call: &Call,
    owner: Option<&ModuleBinding>,
    parts: &[String],
    already_selected: bool,
) -> bool {
    if parts.len() <= 1
        || parts.last().is_none_or(|name| name != "query")
        || !profile.executor_names.iter().any(|name| name == "query")
    {
        return false;
    }
    profile.import_specifier.is_empty()
        || (call.unproven_member
            && !already_selected
            && owner.is_some_and(|binding| selection.advisory_receivers.contains(&binding.id)))
}

pub(super) fn alias_target(
    syntax: &Syntax,
    selection: &Selection,
    root: &Span,
    parts: &[String],
) -> bool {
    (parts.len() == 1 || (parts.len() == 2 && parts[1] == "query"))
        && syntax
            .aliases
            .get(root)
            .is_some_and(|id| selection.selected.contains(id) || selection.typed.contains(id))
}
