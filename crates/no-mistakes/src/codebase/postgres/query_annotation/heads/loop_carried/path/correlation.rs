//! Correlate only stable complementary tests, using finite fixed-value walks.
use super::{Condition, Fixed, Key, Mode, Node, Span, WalkInput};
use crate::fx::FxHashMap;

impl Node {
    pub(in crate::codebase::postgres::query_annotation::heads::loop_carried) fn condition_references(
        &self,
        references: &mut Vec<Span>,
    ) {
        match self {
            Self::Branch(condition, left, right) => {
                if let Some(condition) = condition {
                    references.extend(&condition.references);
                }
                left.condition_references(references);
                right.condition_references(references);
            }
            Self::Sequence(nodes) => {
                for node in nodes {
                    node.condition_references(references);
                }
            }
            Self::Scan(_) | Self::Backedge | Self::Exit => {}
        }
    }

    pub(in crate::codebase::postgres::query_annotation::heads::loop_carried) fn witness<F>(
        &self,
        writes: &[Span],
        assignment: Span,
        read: Span,
        stable: F,
    ) -> bool
    where
        F: Fn(&Condition) -> Option<Vec<usize>>,
    {
        if !self.witness_with(writes, assignment, read, &stable, None) {
            return false;
        }
        let mut masks = FxHashMap::default();
        self.condition_masks(&stable, &mut masks);
        // One fixed key at a time keeps the analysis linear in each test's AST.
        // Suppress only when neither truth value has a full loop witness.
        for ((key, owners), mask) in masks {
            if mask == 3
                && ![false, true].into_iter().any(|value| {
                    self.witness_with(
                        writes,
                        assignment,
                        read,
                        &stable,
                        Some(Fixed {
                            key,
                            owners: &owners,
                            value,
                        }),
                    )
                })
            {
                return false;
            }
        }
        true
    }

    fn witness_with<F>(
        &self,
        writes: &[Span],
        assignment: Span,
        read: Span,
        stable: &F,
        fixed: Option<Fixed<'_>>,
    ) -> bool
    where
        F: Fn(&Condition) -> Option<Vec<usize>>,
    {
        let assigned = self.walk(
            1,
            WalkInput {
                writes,
                anchor: assignment,
                mode: Mode::LiteralToBackedge,
            },
            stable,
            fixed,
            &mut Vec::new(),
        );
        (assigned.hit || assigned.next & 2 != 0)
            && self
                .walk(
                    2,
                    WalkInput {
                        writes,
                        anchor: read,
                        mode: Mode::BackedgeToRead,
                    },
                    stable,
                    fixed,
                    &mut Vec::new(),
                )
                .hit
    }

    fn condition_masks<'a, F>(
        &'a self,
        stable: &F,
        masks: &mut FxHashMap<(&'a Key, Vec<usize>), u8>,
    ) where
        F: Fn(&Condition) -> Option<Vec<usize>>,
    {
        match self {
            Self::Branch(condition, left, right) => {
                if let Some(condition) = condition {
                    if let Some(owners) = stable(condition) {
                        *masks.entry((&condition.key, owners)).or_default() |=
                            if condition.negative { 2 } else { 1 };
                    }
                }
                left.condition_masks(stable, masks);
                right.condition_masks(stable, masks);
            }
            Self::Sequence(nodes) => {
                for node in nodes {
                    node.condition_masks(stable, masks);
                }
            }
            Self::Scan(_) | Self::Backedge | Self::Exit => {}
        }
    }
}
