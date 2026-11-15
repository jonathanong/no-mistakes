use super::collect::collect_named_functions;
use super::*;
pub(super) fn collect(
    program: &Program<'_>,
    source: &str,
    trusted_sql_tags: &[TrustedSqlTag],
    raw_sql_tags: &[TrustedSqlTag],
) -> LocalFunctions {
    let mut raw: HashMap<&str, Resolvable<'_>> = HashMap::new();
    for statement in &program.body {
        collect_named_functions(statement, &mut raw);
    }
    let declared_elsewhere = bodies::collect(program);
    let reassigned = ReassignedNames::collect(program);
    raw.retain(|name, _| !reassigned.contains(name));
    let set_safe = crate::codebase::postgres::embedded::walk::guarded_sql::intrinsic_safe(program);
    let guarded_identifiers = raw
        .iter()
        .filter(|_| set_safe)
        .filter(|(_, function)| {
            !crate::codebase::postgres::embedded::walk::guarded_sql::shadows_set(
                function.params,
                function.body,
            )
        })
        .filter_map(|(name, function)| {
            crate::codebase::postgres::embedded::walk::guarded_sql::GuardedCase::returned_values(
                function.params,
                function.body,
            )
            .map(|values| (name.to_string(), values))
        })
        .collect();
    let sql_statement_types =
        crate::codebase::postgres::embedded::bindings::sql_statement_type_bindings(program);
    let mut tag_shadows = TagShadows::collect(program, trusted_sql_tags, &sql_statement_types);
    tag_shadows.configure_raw(program, raw_sql_tags, &sql_statement_types);
    let templates = template_helpers(program, source, &reassigned);
    let programs = programs::lower_all(
        program,
        source,
        &raw,
        &tag_shadows,
        &reassigned,
        &sql_statement_types,
    );
    let mut resolved = FxHashMap::default();
    let mut parameter_builders = HashMap::new();
    for name in raw.keys().copied() {
        let mut resolving = Vec::new();
        if let Some(value) = resolve_named(
            name,
            source,
            MAX_RESOLVE_DEPTH,
            &Callables {
                raw: &raw,
                templates: &templates,
            },
            &mut resolving,
            &tag_shadows,
        ) {
            resolved.insert(name.to_string(), value);
        }
        if let Some(builder) = parameter::summarize(
            raw[name].params,
            raw[name].body,
            &sql_statement_types,
            &tag_shadows,
            source,
        ) {
            parameter_builders.insert(name.to_string(), builder);
        }
    }
    LocalFunctions {
        resolved,
        guarded_identifiers,
        templates,
        parameter_builders,
        tag_shadows,
        programs,
        declared_elsewhere,
    }
}
