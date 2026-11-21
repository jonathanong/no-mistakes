use super::fact_reads::{unrecorded_static_specifier, window_loads_module};
use super::text::declarator_text;
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression};
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
    ts.value_flow
        .bindings
        .iter()
        .any(|binding| binding_misses_static_load(ts, source, binding))
        || ts.value_flow.calls.iter().any(|call| {
            source
                .get(call.offset as usize..call.end as usize)
                .is_some_and(|window| unrecorded_static_specifier(ts, window, call.offset))
        })
}

fn binding_misses_static_load(ts: &TsFileFacts, source: &str, binding: &ValueBinding) -> bool {
    match &binding.value {
        ValueExpression::Call(start, end) => {
            let Some(window) = source.get(*start as usize..*end as usize) else {
                return false;
            };
            window_loads_module(window) && unrecorded_static_specifier(ts, window, *start)
        }
        _ => {
            let rest = source.get(binding.declared_at as usize..).unwrap_or("");
            let window = declarator_text(rest);
            window_loads_module(window)
                && unrecorded_static_specifier(ts, window, binding.declared_at)
        }
    }
}
