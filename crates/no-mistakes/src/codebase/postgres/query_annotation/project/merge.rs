use super::super::evaluate::{CallSpan, ExecutionEvents};
use super::output::{prefix, projected_sql};
use super::{output, ProjectedCalls, Value};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::{sql_requires_query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::path::PathBuf;

struct CallUpdate {
    path: PathBuf,
    span: CallSpan,
    value: Option<String>,
}

pub(super) struct Projection {
    calls: Vec<CallUpdate>,
    executed: ProjectedCalls,
}

pub(super) fn reduce(events: ExecutionEvents, collect_executed: bool) -> Projection {
    let mut projection = Projection {
        calls: Vec::new(),
        executed: ProjectedCalls::new(),
    };
    for ((path, span), events) in events {
        // An exported helper can be called both locally with known SQL and
        // externally with an unknown argument. The annotation rule keeps
        // its contextual preference; executed SQL must include every
        // reachable invocation before it certifies a complete statement.
        let contextual = !collect_executed && events.iter().any(|(generic, _)| !generic);
        let values = events
            .into_iter()
            .filter(|(generic, _)| !contextual || !generic)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        if values
            .iter()
            .all(|value| matches!(value, Value::Unsupported))
        {
            // Legacy lexical SQL facts remain authoritative where this
            // additive helper projection does not model the scope.
            continue;
        }
        if collect_executed {
            let mut alternatives = Vec::new();
            let mut reason = None;
            for value in &values {
                if let Some(candidates) = projected_sql(value) {
                    for candidate in candidates {
                        if !candidate.complete && !candidate.only_value_placeholders {
                            reason = Some("unresolved-sql-fragment");
                            break;
                        }
                        if !alternatives.contains(&candidate) {
                            alternatives.push(candidate);
                            if alternatives.len() > super::super::super::MAX_EMBEDDED_SQL_VARIANTS {
                                reason = Some("variant-limit");
                                break;
                            }
                        }
                    }
                } else {
                    reason = Some(output::incomplete_reason(value));
                    break;
                }
            }
            projection
                .executed
                .insert((path.clone(), span), reason.map_or(Ok(alternatives), Err));
        }
        let values = values.into_iter().map(prefix).collect::<Vec<_>>();
        // Every analyzable invocation must satisfy the rule. One favorable
        // callback callsite must never hide another unannotated/opaque one.
        let value = values
            .iter()
            .flatten()
            .find(|value| sql_requires_query_annotation(value))
            .cloned()
            .or_else(|| {
                if values.iter().any(Option::is_none) {
                    None
                } else {
                    values.into_iter().flatten().next()
                }
            });
        projection.calls.push(CallUpdate { path, span, value });
    }
    projection
}

pub(super) fn apply(
    files: &mut FileIdMap<CheckFileFacts>,
    originals: &FxHashMap<PathBuf, PathBuf>,
    options: &EmbeddedSqlOptions,
    mut projection: Projection,
    projected: &mut ProjectedCalls,
) {
    for CallUpdate { path, span, value } in projection.calls {
        let file = files
            .get_mut(&originals[&path])
            .expect("projected source remains in the prepared map");
        let (_, facts) = file
            .query_annotation
            .iter_mut()
            .find(|(profile, _)| profile == options)
            .expect("projected annotation profile remains prepared");
        facts.calls.insert(span, value);
    }
    // Later profiles replace earlier values for the same projected call.
    projected.append(&mut projection.executed);
}
