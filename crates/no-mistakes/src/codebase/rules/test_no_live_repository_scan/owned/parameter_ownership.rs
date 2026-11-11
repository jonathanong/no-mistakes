use super::super::cause::{Cause, CauseKind, Ownership};
use super::super::Owners;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin, PathValueKind};

impl Owners<'_> {
    pub(in super::super::super) fn ownership(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Ownership {
        let proof = self.proof_ownership(origin, site, facts, fixture_dirs);
        if matches!(proof, Ownership::No) {
            if let Some(cause) = self.env_cause(origin, site, facts) {
                return Ownership::Environment(cause);
            }
        }
        proof
    }

    fn proof_ownership(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Ownership {
        if origin.require_scalar && origin.value_kind == PathValueKind::Mutable {
            return Ownership::No;
        }
        if !origin.alternatives.is_empty() {
            for atom in origin.atoms() {
                let mut atom = atom.clone();
                atom.require_scalar |= origin.require_scalar;
                use super::super::super::provenance::{classify, Verdict};
                if classify(Some(&atom), fixture_dirs) == Verdict::Allowed {
                    continue;
                }
                let ownership = self.ownership(&atom, site, facts, fixture_dirs);
                if !matches!(ownership, Ownership::Yes) {
                    return ownership;
                }
            }
            return Ownership::Yes;
        }
        if origin.getter.is_some() {
            return if self.getter_owned(origin, site, facts) {
                Ownership::Yes
            } else {
                Ownership::No
            };
        }
        let climbs = origin.segments.iter().any(|segment| segment == "..")
            && (origin.param.is_some()
                || origin.env.is_some()
                || origin.anchor == PathAnchor::Temp);
        if climbs {
            if origin.param.is_some() && origin.env.is_none() {
                return Ownership::Parameter(Cause::new(
                    CauseKind::ClimbsOut,
                    site.file.clone(),
                    site.line,
                    "",
                ));
            }
            return Ownership::No;
        }
        if origin.anchor == PathAnchor::Temp {
            return if origin
                .root_tails
                .iter()
                .all(|tail| self.safe_suffix(tail, site, facts, 0))
            {
                Ownership::Yes
            } else {
                Ownership::No
            };
        }
        match (&origin.env, origin.param) {
            (Some(name), _) => {
                if origin
                    .root_tails
                    .iter()
                    .all(|tail| self.safe_suffix(tail, site, facts, 0))
                    && self.env_owned(name, origin.env_capture, site, facts)
                {
                    Ownership::Yes
                } else {
                    Ownership::No
                }
            }
            (None, Some(index)) => match self.param_cause(index, origin, site, facts, fixture_dirs)
            {
                Ok(()) => Ownership::Yes,
                Err(cause) => Ownership::Parameter(cause),
            },
            (None, None) => origin.reassigned_at.map_or(Ownership::No, |line| {
                Ownership::Parameter(Cause::new(
                    CauseKind::ReassignedParameter,
                    site.file.clone(),
                    line,
                    "",
                ))
            }),
        }
    }
}
