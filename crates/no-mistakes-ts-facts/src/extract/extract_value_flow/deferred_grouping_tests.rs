// Load as a child module of mutable_deferred so these tests exercise real push.
use super::*;
use std::{cell::RefCell, rc::Rc, sync::Arc};

fn identity() -> Rc<RefCell<active_reference_phases::ActiveReferencePhases>> {
    let catalog = Arc::new(phase_catalog::PhaseCatalog {
        start: 0,
        end: 100,
        segments: Vec::new(),
    });
    Rc::new(RefCell::new(
        active_reference_phases::ActiveReferencePhases::new(
            (&catalog, 100),
            &[],
            None,
            &receiver_histories::ReceiverHistories::default(),
            None,
            false,
            phase_walk::ReferencePhaseEvents {
                opaque: None,
                fresh: None,
            },
        ),
    ))
}

fn mutation(
    key: (u32, Option<CallableId>, u64),
    marker: u32,
    phases: &Rc<RefCell<active_reference_phases::ActiveReferencePhases>>,
) -> DeferredMutation {
    DeferredMutation {
        sources: vec![(
            (
                (
                    (marker as usize, format!("source{marker}")),
                    vec![Some("get".into())],
                ),
                vec![Some((marker, Some(CallableId(marker))))],
                (marker + 1, None),
            ),
            None,
            marker.is_multiple_of(2),
        )],
        metadata: MutationMetadata {
            offset: key.0,
            context: key.1,
            epoch: key.2,
            fresh: marker.is_multiple_of(2),
            unconditional: !marker.is_multiple_of(2),
            region: Some(marker),
            phase: Some((marker, marker + 1)),
            foreign_before: marker as usize,
        },
        assigned_member: Some(format!("member{marker}")),
        completed_at: marker + 100,
        has_assignment: marker.is_multiple_of(2),
        proof: DeferredProof {
            phases: Some(Rc::clone(phases)),
            region: Some(marker + 10),
            owner: Some(Some(CallableId(marker + 20))),
        },
    }
}

#[test]
fn nested_merge_and_direct_append_preserve_first_proof_and_source_order() {
    let keys = [
        (10, None, 0),
        (20, None, 0),
        (10, None, 0),
        (10, Some(CallableId(1)), 0),
        (10, None, 1),
        (30, None, 0),
        (10, None, 0),
    ];
    let phases: Vec<_> = (0..keys.len()).map(|_| identity()).collect();
    let make = |index: usize| mutation(keys[index], index as u32, &phases[index]);
    let mut old = DeferredMutations::default();
    old.push(make(0));
    let mut child = DeferredMutations::default();
    child.push(make(1));
    child.push(make(2));
    let mut grandchild = DeferredMutations::default();
    grandchild.push(make(3));
    grandchild.push(make(4));
    // Exact merge operation used by finish_deferred_mutable_writes.
    for entry in grandchild.pending {
        child.push(entry);
    }
    child.push(make(5));
    for entry in child.pending {
        old.push(entry);
    }
    old.push(make(6));
    let mut flat = DeferredMutations::default();
    for index in 0..keys.len() {
        flat.push(make(index));
    }
    assert_eq!(old.indexes, flat.indexes);
    assert_eq!(flat.pending.len(), 5);
    for ((old, new), first) in old.pending.iter().zip(&flat.pending).zip([0, 1, 3, 4, 5]) {
        let expected = make(first);
        for actual in [old, new] {
            assert_eq!(actual.sources, new.sources);
            assert_eq!(actual.metadata.offset, expected.metadata.offset);
            assert_eq!(actual.metadata.context, expected.metadata.context);
            assert_eq!(actual.metadata.epoch, expected.metadata.epoch);
            assert_eq!(actual.metadata.fresh, expected.metadata.fresh);
            assert_eq!(
                actual.metadata.unconditional,
                expected.metadata.unconditional
            );
            assert_eq!(actual.metadata.region, expected.metadata.region);
            assert_eq!(actual.metadata.phase, expected.metadata.phase);
            assert_eq!(
                actual.metadata.foreign_before,
                expected.metadata.foreign_before
            );
            assert_eq!(actual.assigned_member, expected.assigned_member);
            assert_eq!(actual.completed_at, expected.completed_at);
            assert_eq!(actual.has_assignment, expected.has_assignment);
            assert_eq!(actual.proof.region, expected.proof.region);
            assert_eq!(actual.proof.owner, expected.proof.owner);
            assert!(Rc::ptr_eq(
                actual.proof.phases.as_ref().unwrap(),
                &phases[first]
            ));
        }
    }
    let markers: Vec<_> = flat.pending[0]
        .sources
        .iter()
        .map(|entry| entry.0 .0 .0 .0)
        .collect();
    assert_eq!(markers, [0, 2, 6]);
}
