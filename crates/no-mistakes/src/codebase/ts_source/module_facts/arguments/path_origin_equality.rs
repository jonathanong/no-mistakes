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
mod tests {
    use super::*;
    use crate::codebase::ts_source::module_facts::arguments::EnvDiagnostic;
    use std::collections::hash_map::DefaultHasher;

    fn hash(origin: &PathOrigin) -> u64 {
        let mut hasher = DefaultHasher::new();
        origin.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn diagnostic_metadata_does_not_change_proof_identity() {
        let unknown = PathOrigin::unknown();
        let mut diagnostic = unknown.clone();
        diagnostic.env_diagnostic = Some(Box::new(EnvDiagnostic {
            name: "SCAN_ROOT".to_owned(),
            fallback: Some("src".to_owned()),
            captured: true,
            line: 3,
        }));
        diagnostic.captured_getter = Some(42);
        assert_eq!(unknown, diagnostic);
        assert_eq!(hash(&unknown), hash(&diagnostic));
        assert_eq!(
            PathOrigin::alternatives(vec![unknown.clone(), diagnostic]),
            unknown
        );
    }

    #[test]
    fn proof_fields_remain_distinct() {
        let unknown = PathOrigin::unknown();
        let mut environment = unknown.clone();
        environment.env = Some("SCAN_ROOT".to_owned());
        assert_ne!(unknown, environment);
        let mut parameter = unknown.clone();
        parameter.param = Some(0);
        assert_ne!(unknown, parameter);
        let mut scalar = unknown.clone();
        scalar.require_scalar = true;
        assert_ne!(unknown, scalar);
    }
}
