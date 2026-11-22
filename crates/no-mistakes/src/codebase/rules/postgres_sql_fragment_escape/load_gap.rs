use super::fact_reads::{unrecorded_static_specifier, window_loads_module};
use super::modules::module_file;
use super::resolve::FileCtx;
use super::text::declarator_text;
use super::visible::{visible, visible_all};
use crate::codebase::dependencies::extract::{ImportKind, ValueBinding, ValueExpression};
use crate::codebase::ts_source::facts::TsFileFacts;

pub(super) fn missing_static_load_gap(
    ts: &TsFileFacts,
    source: &str,
    relative: &str,
) -> Option<String> {
    has_static_load_miss(ts, source).then(|| {
        format!(
            "`{relative}` has no recorded specifier for a direct `import()` or `require()`, so a write, rebind, or escape of an exported SQL fragment is not recovered from source text. Restore the literal import fact and rerun the check."
        )
    })
}

fn has_static_load_miss(ts: &TsFileFacts, source: &str) -> bool {
    computed_direct_load(ts)
        || ts
            .value_flow
            .bindings
            .iter()
            .any(|binding| binding_misses_static_load(ts, source, binding))
        || ts.value_flow.calls.iter().any(|call| {
            source
                .get(call.offset as usize..call.end as usize)
                .is_some_and(|window| unrecorded_static_specifier(ts, window, call.offset))
        })
}

fn computed_direct_load(ts: &TsFileFacts) -> bool {
    ts.namespaces.computed_created_require
        || ts.imports.iter().any(|import| {
            import.computed && matches!(import.kind, ImportKind::Dynamic | ImportKind::Require)
        })
}

pub(super) fn missing_sourced_export_gaps(ctx: &FileCtx<'_>, relative: &str) -> Vec<String> {
    let mut gaps = Vec::new();
    for export in &ctx.ts.exported_bindings {
        let Some(specifier) = &export.specifier else {
            continue;
        };
        if sourced_offset_recorded(ctx.ts, specifier, &export.local, &export.exported) {
            continue;
        }
        if !sourced_target_hides_fragment(ctx, specifier, &export.local) {
            continue;
        }
        let name = if export.local == "*" {
            export.exported.as_str()
        } else {
            export.local.as_str()
        };
        gaps.push(format!(
            "`{relative}` has no recorded export offset for `{name}`, so a re-export of an imported SQL fragment is not recovered from source text. Restore the namespace export fact and rerun the check."
        ));
    }
    gaps
}

fn sourced_offset_recorded(ts: &TsFileFacts, specifier: &str, local: &str, exported: &str) -> bool {
    ts.namespaces
        .sourced_export_offsets
        .iter()
        .any(|(spec, loc, exp, _)| spec == specifier && loc == local && exp == exported)
}

fn sourced_target_hides_fragment(ctx: &FileCtx<'_>, specifier: &str, local: &str) -> bool {
    let Some(module) = module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, specifier) else {
        return false;
    };
    if local == "*" {
        return !visible_all(ctx.catalog, ctx.graph, ctx.facts, &module, &mut Vec::new())
            .is_empty();
    }
    !visible(
        ctx.catalog,
        ctx.graph,
        ctx.facts,
        &module,
        local,
        &mut Vec::new(),
    )
    .is_empty()
}

fn binding_misses_static_load(ts: &TsFileFacts, source: &str, binding: &ValueBinding) -> bool {
    match &binding.value {
        ValueExpression::Call(start, end) => {
            let Some(window) = source.get(*start as usize..*end as usize) else {
                return false;
            };
            window_loads_module(ts, window, *start)
                && unrecorded_static_specifier(ts, window, *start)
        }
        _ => {
            let rest = source.get(binding.declared_at as usize..).unwrap_or("");
            let window = declarator_text(rest);
            window_loads_module(ts, window, binding.declared_at)
                && unrecorded_static_specifier(ts, window, binding.declared_at)
        }
    }
}
