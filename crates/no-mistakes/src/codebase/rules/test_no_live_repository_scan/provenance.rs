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
    if origin.anchor == PathAnchor::Temp {
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
        PathAnchor::Temp | PathAnchor::Unknown => Verdict::Unresolved(
            "the root is a parameter, mutable binding, or expression this analysis cannot follow",
        ),
    }
}
