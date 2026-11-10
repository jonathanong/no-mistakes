//! Message text for one unproven parameter root.
use super::{Cause, CauseKind};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn detail(cause: &Cause, root: &Path) -> String {
    let place = format!("{}:{}", relative_slash_path(root, &cause.file), cause.line);
    let expr = cause.expression.as_str();
    match cause.kind {
        CauseKind::LivePath => passed(&place, expr, "a live path"),
        CauseKind::OmittedArgument => format!("{place} omits the root argument"),
        CauseKind::UnprovenVariable => passed(&place, expr, "an unproven variable"),
        CauseKind::ForwardedParameter => {
            passed(&place, expr, "a value forwarded through another parameter")
        }
        CauseKind::NamespaceImport => route(&place, expr, "a namespace import"),
        CauseKind::DefaultImport => route(&place, expr, "a default import"),
        CauseKind::DynamicImport => route(&place, expr, "a dynamic import()"),
        CauseKind::RequireImport => route(&place, expr, "a require()"),
        CauseKind::ViMock => route(&place, expr, "vi.mock"),
        CauseKind::NonDirectReference => format!(
            "callers are unknowable because {place} references `{expr}` other than as a direct call"
        ),
        CauseKind::NotNamedFunction => {
            format!("callers are unknowable because {place} is not a named function")
        }
        CauseKind::ReassignedParameter => format!("{place} reassigns the parameter"),
        CauseKind::NoCaller => {
            format!("no caller call site passes the parameter ({place} declares it)")
        }
        CauseKind::ClimbsOut => format!("{place} climbs above the caller-supplied root"),
        CauseKind::UnresolvedSetup => more(
            &format!("callers are unknowable because {place} does not resolve setupFiles"),
            cause.extra,
        ),
        CauseKind::UnreadModule => unread(&place, &cause.reached, cause.extra),
        CauseKind::UnreadableConfig => {
            format!("callers are unknowable because {place} cannot be read or does not match these tests")
        }
        CauseKind::RecursiveHelper => format!(
            "callers are unknowable because {place} calls itself through a recursive inner helper"
        ),
        CauseKind::CallersNotProven => format!("callers are not proven ({place} declares it)"),
    }
}

pub(super) fn fix(kind: CauseKind) -> &'static str {
    match kind {
        CauseKind::LivePath | CauseKind::OmittedArgument | CauseKind::UnprovenVariable => {
            "pass a const fixture path or an owned mkdtemp root from that caller"
        }
        CauseKind::ForwardedParameter => {
            "pass an owned root from the caller that supplies that parameter"
        }
        CauseKind::NamespaceImport
        | CauseKind::DefaultImport
        | CauseKind::DynamicImport
        | CauseKind::RequireImport => {
            "import the helper with a named static import and pass an owned root from the caller"
        }
        CauseKind::ViMock => "do not vi.mock this module, and pass an owned root from the caller",
        CauseKind::NonDirectReference => {
            "call the helper directly and pass an owned root from the caller"
        }
        CauseKind::NotNamedFunction => {
            "declare the helper as a named function and pass an owned root from each caller"
        }
        CauseKind::ReassignedParameter => {
            "stop reassigning the parameter and pass an owned root from each caller"
        }
        CauseKind::NoCaller => "pass an owned root from the caller",
        CauseKind::ClimbsOut => {
            "do not traverse above the caller-supplied root; pass an owned root from the caller"
        }
        CauseKind::UnresolvedSetup => {
            "give setupFiles a literal path, or pass an owned root from a caller this analysis can see"
        }
        CauseKind::UnreadModule => {
            "use a module specifier this analysis can read, and pass an owned root from the caller"
        }
        CauseKind::UnreadableConfig => {
            "make the Vitest config readable and match the selected tests, or pass an owned root from a caller this analysis can see"
        }
        CauseKind::RecursiveHelper => {
            "pass an owned root into the recursive helper from a caller this analysis can see"
        }
        CauseKind::CallersNotProven => "pass an owned root from a caller this analysis can see",
    }
}

fn unread(place: &str, reached: &str, extra: u32) -> String {
    let mut text =
        format!("callers are unknowable because {place} loads a module this analysis cannot read");
    if !reached.is_empty() {
        text.push_str(", reached from ");
        text.push_str(reached);
    }
    more(&text, extra)
}

fn more(text: &str, extra: u32) -> String {
    if extra == 0 {
        text.to_string()
    } else {
        format!("{text}, and {extra} more")
    }
}

fn passed(place: &str, expr: &str, why: &str) -> String {
    if expr.is_empty() {
        format!("{place} passes an argument that is {why}")
    } else {
        format!("{place} passes `{expr}`, {why}")
    }
}

fn route(place: &str, specifier: &str, form: &str) -> String {
    if specifier.is_empty() {
        format!("callers are unknowable because {place} uses {form}")
    } else {
        format!("callers are unknowable because {place} imports `{specifier}` through {form}")
    }
}

#[cfg(test)]
#[path = "cause_text/tests.rs"]
mod tests;
