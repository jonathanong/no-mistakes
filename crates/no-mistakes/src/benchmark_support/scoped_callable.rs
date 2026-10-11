use super::call_index::{callable_file_index_fixture, CallableFileIndexFixture};
use crate::codebase::dependencies::extract::CallableId;
pub use crate::codebase::dependencies::graph::BenchmarkCallableIndex;

/// Same binding demand with either one shared lexical scope or many small scopes.
pub struct ScopedCallableFixture {
    fixture: CallableFileIndexFixture,
    pub scope: usize,
    pub deep_scope: usize,
    pub binding: String,
    pub alias: String,
    pub class_scope: usize,
}

/// Prepare dense or sparse lexical bindings, plus a sixteen-frame parent chain.
pub fn scoped_callable_fixture(entries: usize, sparse: bool) -> ScopedCallableFixture {
    let mut fixture = callable_file_index_fixture(entries);
    let facts = &mut fixture.facts;
    let classes = facts.class_scopes.len();
    if sparse {
        for (index, (scope, name, _)) in facts.callable_bindings.iter_mut().enumerate() {
            *scope = index + 1;
            if index < entries {
                *name = "fn".to_string();
                facts.callable_scope_ids[index].1 = "fn".to_string();
                facts.callable_scopes[index] = "fn".to_string();
            }
        }
        for (index, alias) in facts.callable_aliases.iter_mut().enumerate() {
            alias.binding_scope = index + 1;
            alias.local = "invoke".to_string();
            alias.target = "fn".to_string();
        }
        for (index, call) in facts.function_calls.iter_mut().enumerate() {
            call.callee_binding_scope = Some(index + 1);
            if index < entries {
                call.callee = "invoke".to_string();
            }
        }
        facts
            .lexical_scope_parents
            .extend((1..=entries + classes).map(|scope| (scope, Some(0))));
    }
    facts.lexical_scope_parents.push((0, None));
    let scope = usize::from(sparse);
    let mut parent = scope;
    for depth in 0..16 {
        let child = entries + classes + depth + 1;
        facts.lexical_scope_parents.push((child, Some(parent)));
        let id = CallableId(3_000_000 + depth as u32);
        let display = format!("nested-{depth}/other");
        facts.callable_scopes.push(display.clone());
        facts.callable_scope_ids.push((id, display));
        facts
            .callable_bindings
            .push((child, "other".to_string(), id));
        parent = child;
    }
    facts.callable_binding_declared_at = facts
        .callable_bindings
        .iter()
        .map(|(scope, name, _)| (*scope, name.clone(), 0))
        .collect();
    ScopedCallableFixture {
        fixture,
        scope,
        deep_scope: parent,
        binding: if sparse { "fn" } else { "fn0" }.to_string(),
        alias: if sparse { "invoke" } else { "alias0" }.to_string(),
        class_scope: if sparse { entries + 1 } else { 0 },
    }
}

/// Constructor-only projection used by the dense/sparse control benchmarks.
pub fn construct_scoped_callable_index(fixture: &ScopedCallableFixture) -> usize {
    crate::codebase::dependencies::graph::benchmark_construct_callable_file_index(
        &fixture.fixture.facts,
    )
}

/// Prepare once; all subsequent probes borrow the index and input names.
pub fn prepare_scoped_callable_index(fixture: &ScopedCallableFixture) -> BenchmarkCallableIndex {
    crate::codebase::dependencies::graph::benchmark_prepare_callable_index(&fixture.fixture.facts)
}
