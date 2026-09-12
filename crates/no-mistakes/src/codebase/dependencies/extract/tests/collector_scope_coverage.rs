use super::*;

fn program_collector() -> ImportCollector {
    let mut collector = ImportCollector::default();
    collector.local_stack.push(fx_set());
    collector.lexical_scope_ids.push(0);
    collector.lexical_scope_parents.insert(0, None);
    collector.next_lexical_scope_id = 1;
    collector
}

fn membership(owner: u32, member: &str) -> FunctionCall {
    FunctionCall {
        caller: Some("owner".to_string()),
        caller_id: Some(CallableId(owner)),
        syntactic_caller: None,
        callee: member.to_string(),
        line: 0,
        offset: 0,
        is_callback: true,
        invocation: InvocationKind::Membership,
        target_identity: CallTargetIdentity::RepositoryFunction,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn local_function_scope_covers_constructors_aggregates_and_class_cycles() {
    let mut collector = program_collector();
    collector.local_stack[0].insert("Box".to_string());
    collector.local_stack[0].insert("api".to_string());
    collector.local_stack[0].insert("A".to_string());
    collector.local_stack[0].insert("B".to_string());
    collector.record_callable_binding_id("Box", CallableId(1));
    collector.record_callable_binding_id("api", CallableId(10));
    collector.record_callable_binding_id("A", CallableId(2));
    collector.record_callable_binding_id("B", CallableId(3));
    collector
        .callable_scope_ids
        .insert((CallableId(1), "Box".to_string()));
    collector
        .callable_scope_ids
        .insert((CallableId(2), "A".to_string()));
    collector
        .callable_scope_ids
        .insert((CallableId(3), "B".to_string()));
    collector.class_scopes.insert("Box".to_string());
    collector.class_scopes.insert("A".to_string());
    collector.class_scopes.insert("B".to_string());
    collector
        .class_local_bases
        .insert(CallableId(2), "B".to_string());
    collector
        .class_local_bases
        .insert(CallableId(3), "A".to_string());
    collector.function_calls.push(membership(10, "load"));

    assert!(
        !collector.has_local_function_scope("Box.constructor"),
        "constructor is not a static class member"
    );
    assert!(
        collector.has_local_function_scope("api.load"),
        "object aggregates keep membership calls as local scopes"
    );
    assert!(
        !collector.has_class_static_member(CallableId(2), 0, "missing"),
        "cyclic class bases must not loop"
    );
}

#[test]
fn class_static_members_walk_parent_scopes_and_reject_non_class_bases() {
    let mut collector = program_collector();
    collector.local_stack.push(fx_set());
    collector.lexical_scope_ids.push(1);
    collector.lexical_scope_parents.insert(1, Some(0));
    collector.local_stack[0].insert("Base".to_string());
    collector.local_stack[1].insert("Child".to_string());
    collector.record_callable_binding_id("Child", CallableId(20));
    collector.insert_callable_binding_at(0, "Base".to_string(), CallableId(10));
    collector.insert_callable_binding_name_at(0, "Base".to_string());
    collector
        .callable_scope_ids
        .insert((CallableId(10), "Base".to_string()));
    collector
        .callable_scope_ids
        .insert((CallableId(20), "Child".to_string()));
    collector.class_scopes.insert("Base".to_string());
    collector.class_scopes.insert("Child".to_string());
    collector
        .class_member_callable_ids
        .entry(CallableId(10))
        .or_default()
        .insert("run".to_string(), CallableId(11));
    collector
        .class_local_bases
        .insert(CallableId(20), "Base".to_string());

    assert!(collector.has_class_static_member(CallableId(20), 1, "run"));

    collector
        .class_local_bases
        .insert(CallableId(20), "helper".to_string());
    collector.insert_callable_binding_at(0, "helper".to_string(), CallableId(99));
    collector.insert_callable_binding_name_at(0, "helper".to_string());
    collector
        .callable_scope_ids
        .insert((CallableId(99), "helper".to_string()));
    assert!(!collector.has_class_static_member(CallableId(20), 1, "run"));
}

#[test]
fn class_id_for_binding_follows_aliases_and_stops_on_cycles() {
    let mut collector = program_collector();
    collector.local_stack[0].insert("Alias".to_string());
    collector.local_stack[0].insert("Box".to_string());
    collector.record_callable_binding_id("Box", CallableId(1));
    collector
        .callable_scope_ids
        .insert((CallableId(1), "Box".to_string()));
    collector.class_scopes.insert("Box".to_string());
    collector.callable_aliases.push(CallableAliasBinding {
        alias: CallableAlias {
            scope: None,
            scope_id: None,
            local: "Alias".to_string(),
            target: "Box".to_string(),
            binding_scope: 0,
            declared_at: 0,
            invalidated_at: None,
        },
        lexical_scope_depth: 0,
    });
    collector.callable_alias_index.resize_with(1, fx_map);
    collector.callable_alias_index[0].insert("Alias".to_string(), 0);
    assert_eq!(
        collector.class_id_for_binding(0, "Alias"),
        Some(CallableId(1))
    );

    collector.callable_aliases[0].alias.target = "Alias".to_string();
    collector.callable_alias_index[0].insert("Loop".to_string(), 0);
    collector.callable_aliases[0].alias.local = "Loop".to_string();
    collector.callable_aliases.push(CallableAliasBinding {
        alias: CallableAlias {
            scope: None,
            scope_id: None,
            local: "Loop".to_string(),
            target: "Loop".to_string(),
            binding_scope: 0,
            declared_at: 0,
            invalidated_at: None,
        },
        lexical_scope_depth: 0,
    });
    collector.callable_alias_index[0].insert("Loop".to_string(), 1);
    assert!(collector.class_id_for_binding(0, "Loop").is_none());

    collector.local_stack[0].insert("plain".to_string());
    collector.record_callable_binding_id("plain", CallableId(50));
    assert!(!collector.has_local_function_scope("plain.missing"));
    assert!(!collector.has_local_function_scope("ghost.member"));
}
