//! Keep executable bodies only for catalog providers or replayable sink callers.
use super::{FileModels, Span};
use crate::codebase::{
    dependencies::extract::CallableId,
    postgres::{
        embedded::helpers::program::Stmt,
        sql_identifier_syntax::{FileFacts, Input, SqlCall},
    },
    ts_source::module_facts::TypeScriptModuleFacts,
};
use crate::fx::FxHashSet;
use std::sync::Arc;

impl FileModels {
    pub(in crate::codebase::postgres::sql_identifier_syntax) fn trim_bodies(
        &mut self,
        module: &TypeScriptModuleFacts,
        output: &FileFacts,
    ) {
        if !module.diagnostics.is_empty() || module.exports.iter().any(|export| !export.type_only) {
            return;
        }
        let named = self
            .binding_functions
            .values()
            .filter_map(|id| *id)
            .collect::<FxHashSet<_>>();
        let mut assigned = self
            .locals
            .values()
            .map(|local| local.initializer)
            .collect::<Vec<Span>>();
        assigned.sort_unstable();
        // A prefix maximum also covers arrows wrapped by `as` or parentheses.
        let mut furthest = 0;
        for (_, end) in &mut assigned {
            furthest = furthest.max(*end);
            *end = furthest;
        }
        let mut callers = FxHashSet::<CallableId>::default();
        for (profile, calls) in &output.profiles {
            let Some(sinks) = self.sinks.get(profile) else {
                continue;
            };
            for call in calls {
                if !sinks.contains(&call.span) || !needs_replay(call) {
                    continue;
                }
                if let Ok(index) = self
                    .calls
                    .binary_search_by_key(&call.span, |model| model.span)
                {
                    if let Some(caller) = self.calls[index].caller {
                        callers.insert(caller);
                    }
                }
            }
        }
        for function in self.functions.values_mut() {
            let owner = assigned.partition_point(|(start, _)| *start <= function.span.0);
            let in_initializer = owner > 0 && assigned[owner - 1].1 >= function.span.1;
            if function.binding.is_some()
                || named.contains(&function.id)
                || in_initializer
                || callers.contains(&function.id)
            {
                continue;
            }
            Arc::make_mut(&mut function.lowered.program).body = vec![Stmt::Opaque];
        }
    }
}

fn needs_replay(call: &SqlCall) -> bool {
    match &call.input {
        Input::Unknown => true,
        Input::Sql {
            holes,
            parameterized: None,
            ..
        } => !holes.is_empty(),
        _ => false,
    }
}
