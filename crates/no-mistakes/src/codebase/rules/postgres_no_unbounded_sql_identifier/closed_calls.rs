//! `.join`, `.map`, `.filter`, and `.slice` over a closed string array.
use super::*;
use crate::codebase::dependencies::extract::{
    unique_closed_collection, CallableId, RegistryKind, ValueCall, ValueFlowFacts,
};
use crate::codebase::rules::postgres_sql_cross_file::literal_array;

/// A local binding whose value is built only from a closed array.
pub(super) fn proven(
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    provenance: &Provenance,
) -> bool {
    let Provenance::Reference {
        name,
        scope,
        offset,
        ..
    } = provenance
    else {
        return false;
    };
    let Some(binding) = bindings.value(&call.file, *scope, name, *offset) else {
        return false;
    };
    let ValueExpression::Call(start, end) = binding.value else {
        return false;
    };
    let ctx = Context {
        graph,
        facts,
        bindings,
        file: &call.file,
    };
    ctx.closed_call(start, end)
}

struct Context<'a, 'b> {
    graph: &'a DepGraph,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'b>,
    file: &'a Path,
}

impl Context<'_, '_> {
    fn closed_call(&self, start: u32, end: u32) -> bool {
        let Some(flow) = self.facts.get_ts_facts(self.file).map(|ts| &ts.value_flow) else {
            return false;
        };
        let Some(value_call) = flow
            .calls
            .iter()
            .find(|item| item.offset == start && item.end == end)
        else {
            return false;
        };
        let ValueExpression::Member(base, method) = &value_call.callee else {
            return false;
        };
        match method.as_str() {
            "join" => self.closed_list(base),
            "map" => self.closed_list(base) && closed_callback(flow, value_call),
            "filter" | "slice" => self.closed_list(base),
            _ => false,
        }
    }

    fn closed_list(&self, expr: &ValueExpression) -> bool {
        match expr {
            ValueExpression::Named { name, .. } => self.closed_array(name),
            ValueExpression::Call(start, end) => self.closed_call(*start, *end),
            _ => false,
        }
    }

    fn closed_array(&self, name: &str) -> bool {
        let Some(flow) = self.facts.get_ts_facts(self.file).map(|ts| &ts.value_flow) else {
            return false;
        };
        if unique_closed_collection(flow, name, RegistryKind::Array) {
            return true;
        }
        let Some(unique) = unique_export(
            self.graph,
            self.facts,
            &self.bindings.routes,
            self.file,
            name,
        ) else {
            return false;
        };
        unique.producer != self.file
            && self
                .facts
                .get_ts_facts(&unique.producer)
                .is_some_and(|ts| literal_array(&ts.value_flow, &unique.exported))
    }
}

/// The callback returns only its own parameters, members of them, and literals.
fn closed_callback(flow: &ValueFlowFacts, value_call: &ValueCall) -> bool {
    let Some(ValueExpression::Callable(id)) = value_call.arguments.first() else {
        return false;
    };
    let mut returns = flow
        .returns
        .iter()
        .filter(|(owner, _)| owner == id)
        .peekable();
    returns.peek().is_some() && returns.all(|(_, value)| closed_return(flow, *id, value))
}

fn closed_return(flow: &ValueFlowFacts, id: CallableId, value: &ValueExpression) -> bool {
    match value {
        ValueExpression::Unknown => true,
        ValueExpression::Derived(parts) => parts.iter().all(|part| closed_return(flow, id, part)),
        ValueExpression::Member(inner, _) => closed_return(flow, id, inner),
        ValueExpression::Named { name, .. } => flow
            .parameters
            .iter()
            .any(|parameter| parameter.owner == id && parameter.name == *name),
        _ => false,
    }
}
