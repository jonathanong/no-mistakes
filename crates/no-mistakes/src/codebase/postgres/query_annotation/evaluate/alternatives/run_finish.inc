{
        self.scopes[..scopes.len()].clone_from_slice(&scopes);
        self.active_callback_functions = joined_seen;
        self.modules = modules;
        module_states.restore(&mut self.scopes, &self.modules);
        self.argument_extra_slots = joined_extras.unwrap_or(extra_slots);
        self.restore_builder_updates(joined_updates, updates, original_ids);
        self.remove_alternative_captured_readers(scopes.len());
        if let Some(captured) = captured {
            self.captured_bindings
                .retain(|frame, _| *frame >= scopes.len());
            self.captured_bindings.extend(captured);
        }
        self.add_alternative_captured_readers(scopes.len());
        self.rebuild_mapped_argument_owners();
        if let Some(joined) = joined_scopes {
            self.scopes[..scopes.len()].clone_from_slice(&joined);
        }
        super::freshness::restore(
            &mut self.fresh_mapped_parameters,
            scopes.len(),
            &fresh_joined,
        );
        self.rebuild_fresh_mapped_argument_bindings();
        self.invalidated_builders = original;
        self.deleted_argument_slots = deleted_changed;
        // Recreation inside one arm must not make a deleted property in
        // another arm look live. A write after the branch records its own.
        self.recreated_argument_slots = recreated;
        let mut final_definite = definite_common.unwrap_or(definite.clone());
        final_definite.extend(private_definite);
        // These identities already represent actual arm effects. Replaying an
        // arguments escape would wrongly taint values assigned after that escape.
        self.invalidated_builders.extend(changed.iter().copied());
        super::values::apply_taint(&mut self.scopes, &changed);
        // Restored bindings must retain any prior opaque parameter update before
        // the common deletion disconnects them from future argument mutations.
        self.install_disconnected_slots(&disconnected, disconnected_common, private_disconnected);
        self.definite_deleted_argument_slots = final_definite;
        self.refresh_captured_bindings();
        // Speculative identities with no surviving aliases cannot affect later reads.
        self.active_module_initials
            .pop()
            .expect("balanced alternatives");
        self.prune_final_alternative_state(&returned, final_live);
        self.untrusted_array_intrinsics = joined_untrusted;
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        super::super::concat::alternative_result(returned, precise_records)
}
