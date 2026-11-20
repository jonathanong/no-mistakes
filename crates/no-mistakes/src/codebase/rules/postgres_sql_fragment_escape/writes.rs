use super::fragments::Fragment;
use super::names::namespace_of;
use super::resolve::{name_fragments, FileCtx};
use super::visible::visible;
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
    let mut path = Vec::new();
    collect_path(ctx, name, &mut path, history, sites);
}

fn collect_path(
    ctx: &FileCtx<'_>,
    name: &str,
    path: &mut Vec<String>,
    history: &ValueMutationHistory,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    if let Some(assigned) = &history.assigned {
        push_assigned(ctx, name, path, assigned, sites);
    }
    for (member, child) in &history.members {
        path.push(member.clone());
        collect_path(ctx, name, path, child, sites);
        path.pop();
    }
    if let Some(dynamic) = &history.dynamic {
        collect_path(ctx, name, path, dynamic, sites);
    }
}

fn push_assigned(
    ctx: &FileCtx<'_>,
    name: &str,
    path: &[String],
    assigned: &ValueAssignedMutations,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    for mutation in assigned.members.values() {
        push_mutation(ctx, name, path, mutation, sites);
    }
    if let Some(dynamic) = &assigned.dynamic {
        push_mutation(ctx, name, path, dynamic, sites);
    }
}

fn push_mutation(
    ctx: &FileCtx<'_>,
    name: &str,
    path: &[String],
    mutation: &ValueAssignedMutation,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    push_site(ctx, name, path, mutation.offset, mutation.writer, sites);
}

fn push_site(
    ctx: &FileCtx<'_>,
    name: &str,
    path: &[String],
    offset: u32,
    writer: ValueAssignedWriter,
    sites: &mut Vec<(u32, Vec<Fragment>)>,
) {
    let context = match writer {
        ValueAssignedWriter::Callable(id) => Some(id),
        ValueAssignedWriter::Module | ValueAssignedWriter::Mixed => None,
    };
    let local = FileCtx { context, ..*ctx };
    let fragments = fragments_at(&local, name, path);
    if !fragments.is_empty() {
        sites.push((offset, fragments));
    }
}

/// A write through `ns.FRAG.field` names `FRAG`. A write of the namespace
/// binding itself still names every exported fragment.
fn fragments_at(ctx: &FileCtx<'_>, name: &str, path: &[String]) -> Vec<Fragment> {
    if let Some(member) = path.first() {
        if let Some(module) = namespace_of(ctx, name, 0) {
            return visible(
                ctx.catalog,
                ctx.graph,
                ctx.facts,
                &module,
                member,
                &mut Vec::new(),
            );
        }
    }
    name_fragments(ctx, name)
}
