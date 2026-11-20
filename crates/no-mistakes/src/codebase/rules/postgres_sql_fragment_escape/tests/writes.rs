use super::super::writes::assignment_sites;
use super::support::{catalog, ctx, world};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{
    CallableId, ValueAssignedMutation, ValueAssignedMutations, ValueAssignedWriter,
    ValueMutationHistory,
};
use crate::fx::fx_map;
use std::sync::Arc;

fn mutation(offset: u32, writer: ValueAssignedWriter) -> ValueAssignedMutation {
    ValueAssignedMutation {
        offset,
        unconditional_offset: None,
        writer,
    }
}

fn assigned(
    member: ValueAssignedMutation,
    dynamic: Option<ValueAssignedMutation>,
) -> ValueAssignedMutations {
    let mut members = fx_map();
    members.insert("item".to_string(), member);
    ValueAssignedMutations {
        members,
        dynamic,
        ..ValueAssignedMutations::default()
    }
}

#[test]
fn assignment_history_follows_members_and_dynamic_writes() {
    let mut world = world();
    let escape = world.escape.clone();
    {
        let entry = world.facts.ts.get_mut(&escape).expect("escape facts");
        let mut owned = (*entry.ts).clone();
        let child = ValueMutationHistory {
            assigned: Some(Box::new(assigned(
                mutation(4, ValueAssignedWriter::Callable(CallableId(1))),
                Some(mutation(8, ValueAssignedWriter::Module)),
            ))),
            ..ValueMutationHistory::default()
        };
        let dynamic = ValueMutationHistory {
            assigned: Some(Box::new(assigned(
                mutation(12, ValueAssignedWriter::Mixed),
                None,
            ))),
            ..ValueMutationHistory::default()
        };
        let mut members = fx_map();
        members.insert("item".to_string(), child);
        let root = ValueMutationHistory {
            assigned: Some(Box::new(assigned(
                mutation(16, ValueAssignedWriter::Module),
                None,
            ))),
            members,
            dynamic: Some(Box::new(dynamic)),
            ..ValueMutationHistory::default()
        };
        let mut by_name = fx_map();
        by_name.insert("columns".to_string(), Arc::new(root));
        let mut receivers = fx_map();
        receivers.insert(0, by_name);
        owned.value_flow.receiver_mutations = Some(Arc::new(receivers));
        *entry = Arc::new(CheckFileFacts {
            ts: Arc::new(owned),
            ..CheckFileFacts::default()
        });
    }

    let catalog = catalog(&world);
    let ctx = ctx(&world, &catalog, &world.escape);
    let sites = assignment_sites(&ctx);
    let mut offsets: Vec<u32> = sites.iter().map(|(offset, _)| *offset).collect();
    offsets.sort_unstable();
    assert_eq!(offsets, vec![4_u32, 8, 12, 16]);
    assert!(sites.iter().all(|(_, fragments)| {
        fragments.iter().any(|fragment| {
            fragment.exported == "widgetColumns" && fragment.producer == world.columns
        })
    }));
}
