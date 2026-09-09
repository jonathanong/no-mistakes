fn scope_map_get<'a, V>(
    maps: &'a [FxHashMap<String, V>],
    scope: usize,
    name: &str,
) -> Option<&'a V> {
    maps.get(scope)?.get(name)
}

fn scope_map_insert<V>(
    maps: &mut Vec<FxHashMap<String, V>>,
    scope: usize,
    name: String,
    value: V,
) -> Option<V> {
    if maps.len() <= scope {
        maps.resize_with(scope + 1, fx_map);
    }
    maps[scope].insert(name, value)
}

fn scope_set_contains(sets: &[FxHashSet<String>], scope: usize, name: &str) -> bool {
    sets.get(scope).is_some_and(|set| set.contains(name))
}

fn scope_set_insert(sets: &mut Vec<FxHashSet<String>>, scope: usize, name: String) -> bool {
    if sets.len() <= scope {
        sets.resize_with(scope + 1, fx_set);
    }
    sets[scope].insert(name)
}

fn owner_member_id(
    map: &FxHashMap<CallableId, FxHashMap<String, CallableId>>,
    owner: CallableId,
    member: &str,
) -> Option<CallableId> {
    map.get(&owner)?.get(member).copied()
}

fn owner_member_insert(
    map: &mut FxHashMap<CallableId, FxHashMap<String, CallableId>>,
    owner: CallableId,
    member: String,
    member_id: CallableId,
) -> Option<CallableId> {
    map.entry(owner)
        .or_insert_with(fx_map)
        .insert(member, member_id)
}

fn owner_name_contains(
    map: &FxHashMap<CallableId, FxHashSet<String>>,
    owner: CallableId,
    name: &str,
) -> bool {
    map.get(&owner).is_some_and(|set| set.contains(name))
}

fn owner_name_insert(
    map: &mut FxHashMap<CallableId, FxHashSet<String>>,
    owner: CallableId,
    name: String,
) -> bool {
    map.entry(owner).or_insert_with(fx_set).insert(name)
}

fn flatten_scope_map<V>(maps: Vec<FxHashMap<String, V>>) -> Vec<(usize, String, V)> {
    maps.into_iter()
        .enumerate()
        .flat_map(|(scope, names)| {
            names
                .into_iter()
                .map(move |(name, value)| (scope, name, value))
        })
        .collect()
}

fn flatten_owner_members(
    map: FxHashMap<CallableId, FxHashMap<String, CallableId>>,
) -> Vec<(CallableId, String, CallableId)> {
    map.into_iter()
        .flat_map(|(owner, members)| {
            members
                .into_iter()
                .map(move |(member, id)| (owner, member, id))
        })
        .collect()
}

impl ImportCollector {
    fn callable_binding_at(&self, scope: usize, name: &str) -> Option<CallableId> {
        scope_map_get(&self.callable_bindings, scope, name).copied()
    }

    fn insert_callable_binding_at(&mut self, scope: usize, name: String, id: CallableId) {
        scope_map_insert(&mut self.callable_bindings, scope, name, id);
    }

    fn has_callable_binding_name_at(&self, scope: usize, name: &str) -> bool {
        scope_set_contains(&self.callable_binding_ids, scope, name)
    }

    fn insert_callable_binding_name_at(&mut self, scope: usize, name: String) {
        scope_set_insert(&mut self.callable_binding_ids, scope, name);
    }

    fn has_reassigned_callable_at(&self, scope: usize, name: &str) -> bool {
        scope_set_contains(&self.reassigned_callable_binding_ids, scope, name)
    }

    fn insert_reassigned_callable_at(&mut self, scope: usize, name: String) {
        scope_set_insert(&mut self.reassigned_callable_binding_ids, scope, name);
    }

    fn has_lexical_binding_at(&self, scope: usize, name: &str) -> bool {
        scope_set_contains(&self.lexical_binding_names, scope, name)
    }

    fn insert_lexical_binding_at(&mut self, scope: usize, name: String) {
        scope_set_insert(&mut self.lexical_binding_names, scope, name);
    }

    fn callable_alias_index_at(&self, scope: usize, name: &str) -> Option<usize> {
        scope_map_get(&self.callable_alias_index, scope, name).copied()
    }

    fn insert_callable_alias_index_at(&mut self, scope: usize, name: String, index: usize) {
        scope_map_insert(&mut self.callable_alias_index, scope, name, index);
    }

    fn has_callable_alias_at(&self, scope: usize, name: &str) -> bool {
        self.callable_alias_index_at(scope, name).is_some()
    }

    fn insert_binding_declared_at(&mut self, scope: usize, name: String, offset: u32) {
        scope_map_insert(&mut self.callable_binding_declared_at, scope, name, offset);
    }

    fn class_member_id(&self, class_id: CallableId, member: &str) -> Option<CallableId> {
        owner_member_id(&self.class_member_callable_ids, class_id, member)
    }

    fn has_class_member(&self, class_id: CallableId, member: &str) -> bool {
        self.class_member_id(class_id, member).is_some()
    }

    fn has_class_member_id(
        &self,
        class_id: CallableId,
        member: &str,
        member_id: CallableId,
    ) -> bool {
        self.class_member_id(class_id, member) == Some(member_id)
    }

    fn has_static_getter_member(&self, class_id: CallableId, member: &str) -> bool {
        owner_member_id(&self.static_getter_member_ids, class_id, member).is_some()
    }

    fn has_static_setter_member(&self, class_id: CallableId, member: &str) -> bool {
        owner_member_id(&self.static_setter_member_ids, class_id, member).is_some()
    }

    fn has_object_getter_member(&self, object_id: CallableId, member: &str) -> bool {
        owner_name_contains(&self.object_getter_member_ids, object_id, member)
    }

    fn insert_object_getter_member(&mut self, object_id: CallableId, member: &str) {
        owner_name_insert(
            &mut self.object_getter_member_ids,
            object_id,
            member.to_string(),
        );
    }
}
