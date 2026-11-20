use super::fragments::Fragment;
use super::resolve::{name_fragments, FileCtx};
use crate::codebase::dependencies::extract::{
    ValueAssignedMutation, ValueAssignedMutations, ValueAssignedWriter, ValueMutationHistory,
};

pub(super) fn assignment_sites(ctx: &FileCtx<'_>) -> Vec<(u32, Vec<Fragment>)> {
    let Some(histories) = &ctx.ts.value_flow.receiver_mutations else {
        return Vec::new();
    };
    let mut sites = Vec::new();
    for by_name in histories.values() {
        for (name, history) in by_name {
            collect_history(ctx, name, history, &mut sites);
        }
    }
    sites
}

fn collect_history(
    ctx: &FileCtx<'_>,
    name: &str,
    history: &ValueMutationHistory,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    if let Some(assigned) = &history.assigned {
        push_assigned(ctx, name, assigned, sites);
    }
    for child in history.members.values() {
        collect_history(ctx, name, child, sites);
    }
    if let Some(dynamic) = &history.dynamic {
        collect_history(ctx, name, dynamic, sites);
    }
}

fn push_assigned(
    ctx: &FileCtx<'_>,
    name: &str,
    assigned: &ValueAssignedMutations,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    for mutation in assigned.members.values() {
        push_mutation(ctx, name, mutation, sites);
    }
    if let Some(dynamic) = &assigned.dynamic {
        push_mutation(ctx, name, dynamic, sites);
    }
}

fn push_mutation(
    ctx: &FileCtx<'_>,
    name: &str,
    mutation: &ValueAssignedMutation,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    push_site(ctx, name, mutation.offset, mutation.writer, sites);
}

fn push_site(
    ctx: &FileCtx<'_>,
    name: &str,
    offset: u32,
    writer: ValueAssignedWriter,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    let context = match writer {
        ValueAssignedWriter::Callable(id) => Some(id),
        ValueAssignedWriter::Module | ValueAssignedWriter::Mixed => None,
    };
    let local = FileCtx { context, ..*ctx };
    let fragments = name_fragments(&local, name);
    if !fragments.is_empty() {
        sites.push((offset, fragments));
    }
}
