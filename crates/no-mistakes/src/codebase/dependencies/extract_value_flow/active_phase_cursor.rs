use super::*;

impl<'a> PhaseCursor<'a> {
    pub(in super::super) fn allowed(&self) -> bool {
        self.trusted && self.phases.inert_rhs
    }
    pub(in super::super) fn child(self, member: &str) -> Self {
        Self {
            node: self.node.and_then(|node| node.children.get(member)),
            ..self
        }
    }
    pub(in super::super) fn changed_after_capture(
        &self,
        member: &str,
        capture: &identity_metadata::FreshCapture,
        right: nested::ReceiverRead,
    ) -> bool {
        let Some((offset, owner)) = capture.origin else {
            return true;
        };
        let coordinate = capture
            .phase
            .filter(|(catalog, _)| *catalog == self.phases.catalog.start)
            .map(|(_, phase)| (phase, capture.completed_at))
            .or_else(|| self.phases.coordinate(offset));
        self.changed_from_phase(member, coordinate, owner, right)
    }
    fn changed_from_phase(
        &self,
        member: &str,
        left: Option<(u32, u32)>,
        owner: Option<CallableId>,
        right: nested::ReceiverRead,
    ) -> bool {
        #[cfg(feature = "test-instrumentation")]
        self.phases
            .stats
            .queries
            .set(self.phases.stats.queries.get() + 1);
        let Some((right, other)) = right else {
            return true;
        };
        if !self.allowed() || owner != other || owner != self.phases.owner {
            return true;
        }
        let (Some(left), Some(right)) = (left, self.phases.coordinate(right)) else {
            return true;
        };
        self.phases.effects.as_ref().is_some_and(|effects| {
            left != right
                && effects
                    .range((
                        std::ops::Bound::Excluded(left.min(right)),
                        std::ops::Bound::Included(left.max(right)),
                    ))
                    .next()
                    .is_some()
        }) || identity_metadata::map_between(&self.phases.unresolved, left.0, right.0)
            || self.node.is_some_and(|node| {
                identity_metadata::between(&node.dynamic, left.0, right.0)
                    || node.children.get(member).is_some_and(|child| {
                        identity_metadata::between(&child.writes, left.0, right.0)
                    })
            })
    }
    pub(in super::super) fn changed(
        &self,
        member: &str,
        left: nested::ReceiverRead,
        right: nested::ReceiverRead,
    ) -> bool {
        let Some((left, owner)) = left else {
            return true;
        };
        self.changed_from_phase(member, self.phases.coordinate(left), owner, right)
    }
}
