use super::{PathAnchor, PathDefault, PathOrigin, PathValueKind};
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Hash)]
struct Proof<'a> {
    value_kind: &'a PathValueKind,
    require_scalar: &'a bool,
    relative: &'a bool,
    nullish: &'a bool,
    anchor: &'a PathAnchor,
    segments: &'a Vec<String>,
    dynamic: &'a bool,
    env: &'a Option<String>,
    captured_env: &'a Option<Box<super::CapturedEnv>>,
    getter: &'a Option<u32>,
    root_tails: &'a Vec<PathOrigin>,
    param: &'a Option<usize>,
    param_member: &'a Option<String>,
    param_owner: &'a Option<u32>,
    owner_line: &'a Option<u32>,
    reassigned_at: &'a Option<u32>,
    alternatives: &'a Vec<PathOrigin>,
    param_default: &'a Option<PathDefault>,
}

impl PathOrigin {
    fn proof(&self) -> Proof<'_> {
        let Self {
            value_kind,
            require_scalar,
            relative,
            nullish,
            anchor,
            segments,
            dynamic,
            env,
            captured_env,
            getter,
            root_tails,
            param,
            param_member,
            param_owner,
            owner_line,
            reassigned_at,
            alternatives,
            param_default,
            env_diagnostic: _,
            captured_getter: _,
        } = self;
        Proof {
            value_kind,
            require_scalar,
            relative,
            nullish,
            anchor,
            segments,
            dynamic,
            env,
            captured_env,
            getter,
            root_tails,
            param,
            param_member,
            param_owner,
            owner_line,
            reassigned_at,
            alternatives,
            param_default,
        }
    }
}

impl PartialEq for PathOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.proof() == other.proof()
    }
}

impl Eq for PathOrigin {}

impl Hash for PathOrigin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.proof().hash(state);
    }
}

#[cfg(test)]
#[path = "path_origin_equality_tests.rs"]
mod tests;
