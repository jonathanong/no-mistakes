//! Decide whether an enumeration root is bounded test input.
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// Rooted under a fixtures directory or a temporary directory the test owns.
    Allowed,
    /// Provably rooted in the live repository or working directory.
    Live(&'static str),
    /// The root's provenance could not be established.
    Unresolved(&'static str),
}

pub(super) fn classify(origin: Option<&PathOrigin>, fixture_dirs: &[&str]) -> Verdict {
    let Some(origin) = origin else {
        return Verdict::Unresolved(
            "the path argument was not recorded (spread argument or unparsed module)",
        );
    };
    if !origin.alternatives.is_empty() {
        let mut result = Verdict::Allowed;
        for atom in origin.atoms() {
            match classify(Some(atom), fixture_dirs) {
                verdict @ Verdict::Live(_) => return verdict,
                verdict @ Verdict::Unresolved(_) => result = verdict,
                Verdict::Allowed => {}
            }
        }
        return result;
    }
    if origin.anchor == PathAnchor::Temp && origin.root_tails.is_empty() {
        if origin.segments.iter().any(|segment| segment == "..") {
            return Verdict::Unresolved("the path climbs out of the owned temporary directory");
        }
        return Verdict::Allowed;
    }
    if origin
        .segments
        .iter()
        .any(|segment| fixture_dirs.contains(&segment.as_str()))
    {
        return Verdict::Allowed;
    }
    match origin.anchor {
        PathAnchor::Module => Verdict::Live("a path under the test module's directory"),
        PathAnchor::Cwd => Verdict::Live("a path under process.cwd()"),
        PathAnchor::Relative => {
            Verdict::Live("a relative path resolved against the working directory")
        }
        PathAnchor::Absolute => Verdict::Live("an absolute path outside any fixtures directory"),
        PathAnchor::SharedTemp => {
            Verdict::Live("the shared temporary directory, which the test does not own")
        }
        PathAnchor::Temp | PathAnchor::Unknown => Verdict::Unresolved(
            "the root is a parameter, mutable binding, or expression this analysis cannot follow",
        ),
    }
}

/// Why one caller argument fails the parameter proof. `None` means the walk
/// may still accept it (an owned root, an env root, or a parameter it can follow).
pub(super) enum ArgBlock {
    Live,
    Omitted,
    Variable,
    Forwarded,
}

pub(super) fn missing_block(present: bool) -> ArgBlock {
    match present {
        true => ArgBlock::Variable,
        false => ArgBlock::Omitted,
    }
}

pub(super) fn arg_block(argument: &PathOrigin, fixture_dirs: &[&str]) -> Option<ArgBlock> {
    let climbs = argument.segments.iter().any(|part| part == "..")
        && (argument.param.is_some()
            || argument.env.is_some()
            || argument.anchor == PathAnchor::Temp);
    if climbs {
        return Some(if argument.param.is_some() {
            ArgBlock::Forwarded
        } else {
            ArgBlock::Variable
        });
    }
    if classify(Some(argument), fixture_dirs) == Verdict::Allowed
        || argument.env.is_some()
        || argument.param.is_some()
    {
        return None;
    }
    Some(live_or_variable(argument, fixture_dirs))
}

pub(super) fn live_or_variable(argument: &PathOrigin, fixture_dirs: &[&str]) -> ArgBlock {
    match classify(Some(argument), fixture_dirs) {
        Verdict::Live(_) => ArgBlock::Live,
        _ => ArgBlock::Variable,
    }
}
