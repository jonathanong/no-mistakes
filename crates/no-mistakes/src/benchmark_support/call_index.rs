use crate::codebase::dependencies::extract::{
    CallTargetIdentity, CallableAlias, CallableId, FunctionCall, InvocationKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;

/// Synthetic per-file call facts for measuring `CallableFileIndex::from_facts`.
#[derive(Clone)]
pub struct CallableFileIndexFixture {
    facts: TsFileFacts,
    entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallableFileIndexSummary {
    pub entries: usize,
}

/// Build one dense file: `entries` bindings, aliases, and class members.
pub fn callable_file_index_fixture(entries: usize) -> CallableFileIndexFixture {
    assert!(entries > 0, "entries must be nonzero");
    let class_count = (entries / 16).max(1);
    let members_per_class = 8;
    let mut facts = TsFileFacts::default();
    facts.callable_scopes.reserve(entries + class_count);
    facts.callable_scope_ids.reserve(entries + class_count);
    facts.callable_bindings.reserve(entries + class_count);
    facts.callable_aliases.reserve(entries);
    facts.class_scopes.reserve(class_count);
    facts
        .class_member_callable_ids
        .reserve(class_count * members_per_class);
    facts.function_calls.reserve(entries + class_count);
    for index in 0..entries {
        let name = format!("fn{index}");
        let id = CallableId(index as u32);
        facts.callable_scopes.push(name.clone());
        facts.callable_scope_ids.push((id, name.clone()));
        facts.callable_bindings.push((0, name.clone(), id));
        facts.callable_aliases.push(CallableAlias {
            scope: None,
            scope_id: None,
            local: format!("alias{index}"),
            target: name,
            binding_scope: 0,
            declared_at: 0,
            invalidated_at: None,
        });
        facts.function_calls.push(FunctionCall {
            caller: None,
            caller_id: None,
            syntactic_caller: None,
            callee: format!("alias{index}"),
            line: index as u32 + 1,
            offset: index as u32,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: CallTargetIdentity::Unknown,
            callee_binding_scope: Some(0),
            static_arg: None,
            static_cwd: None,
        });
    }
    for class in 0..class_count {
        let name = format!("Class{class}");
        let class_id = CallableId(1_000_000 + class as u32);
        facts.class_scopes.push(name.clone());
        facts.callable_scopes.push(name.clone());
        facts.callable_scope_ids.push((class_id, name.clone()));
        facts.callable_bindings.push((0, name, class_id));
        for member in 0..members_per_class {
            facts.class_member_callable_ids.push((
                class_id,
                format!("m{member}"),
                CallableId(2_000_000 + (class * members_per_class + member) as u32),
            ));
        }
        facts.function_calls.push(FunctionCall {
            caller: Some(format!("Class{class}")),
            caller_id: Some(class_id),
            syntactic_caller: Some(format!("Class{class}")),
            callee: "Base".to_string(),
            line: 1,
            offset: 0,
            is_callback: true,
            invocation: InvocationKind::Construct,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: Some(0),
            static_arg: None,
            static_cwd: None,
        });
    }
    CallableFileIndexFixture { facts, entries }
}

pub fn construct_callable_file_index(
    fixture: &CallableFileIndexFixture,
) -> CallableFileIndexSummary {
    let constructed = crate::codebase::dependencies::graph::benchmark_construct_callable_file_index(
        &fixture.facts,
    );
    CallableFileIndexSummary {
        entries: constructed.max(fixture.entries),
    }
}

pub fn probe_call_site_files(file_count: usize) -> usize {
    crate::codebase::dependencies::graph::benchmark_probe_call_site_files(file_count)
}
