use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::{Path, PathBuf};

fn push(
    cases: &mut Vec<(String, PathBuf, PathBuf)>,
    rule: &str,
    config: &Path,
    scenarios: &[&str],
) {
    for scenario in scenarios {
        cases.push((
            scenario.replace('-', "_"),
            rules_path(rule, "fixture", scenario),
            config.to_path_buf(),
        ));
    }
}

#[test]
fn postgres_no_unbounded_sql_identifier_check_json_snapshot() {
    let rule = "postgres-no-unbounded-sql-identifier";
    let config = rules_path(rule, "configs", "registry.yml");
    let mut owned = Vec::new();
    // Fixture configs that only differ by effort are not these cases.
    // A finding file ends with query.ts and the message contains
    // "finite-domain validator".
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-env",
            "fail-imported-config",
            "fail-type-assertion",
            "fail-identity-wrapper",
            "fail-to-lowercase",
            "fail-bracket-env",
            "fail-imported-env",
            "fail-imported-process",
            "fail-request-params",
            "fail-db-rows",
        ],
    );
    push(
        &mut owned,
        rule,
        &config,
        &[
            "pass-switch",
            "pass-bind-value",
            "pass-http-query",
            "pass-suppressed",
            "pass-imported-literal",
        ],
    );
    // One finding each, file src/query.ts, message contains
    // "finite-domain validator". Lines and targets:
    // fail-conditional-env line 3 process.env.SCHEMA;
    // fail-template-validator line 6 process.env.SCHEMA;
    // fail-concat-env line 2 process.env.SCHEMA;
    // fail-alias-chain line 4 b;
    // fail-env-object-alias line 3 process.env.SCHEMA;
    // fail-process-namespace line 3 process.env.SCHEMA;
    // fail-destructured-rows line 3 rows;
    // fail-destructured-row line 4 schema;
    // fail-array-destructured-row line 5 id;
    // fail-array-destructured-row-elision line 4 schema;
    // fail-comma-env line 3 process.env.SCHEMA;
    // fail-iterated-rows line 4 row;
    // fail-iterated-row-alias line 4 row.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-conditional-env",
            "fail-template-validator",
            "fail-concat-env",
            "fail-alias-chain",
            "fail-env-object-alias",
            "fail-process-namespace",
            "fail-destructured-rows",
            "fail-destructured-row",
            "fail-array-destructured-row",
            "fail-array-destructured-row-elision",
            "fail-comma-env",
            "fail-iterated-rows",
            "fail-iterated-row-alias",
        ],
    );
    push(
        &mut owned,
        rule,
        &config,
        &[
            "pass-conditional-literals",
            "pass-conditional-validator-local",
            "pass-conditional-validator-nested",
            "pass-conditional-validator-imported",
            "pass-new-bind-value",
            "pass-projected-http-rows",
            "pass-array-destructured-http-row",
            "pass-array-destructured-mutable-row",
            "pass-array-destructured-mutable-result",
            "pass-empty-and-computed-patterns",
            "pass-comma-discarded-env",
            "pass-alias-cycle",
            "pass-alias-shadow",
            "pass-finite-alias",
            "pass-process-namespace-shadow",
            "pass-env-alias-process-parameter",
        ],
    );
    // One finding, file src/query.ts. Lines and targets:
    // untouched-member 5 first; earlier-read 4 first; copied-member 6 schema;
    // env-write 5 first; conditional-write 6 first; alias-write-barrier 7 first;
    // call-barrier 7 first; object-write-barrier 6 first;
    // interpolation-effect-barrier 6 first; shadow-scope 9 first.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-array-row-untouched-member",
            "fail-array-row-earlier-read",
            "fail-array-row-copied-member",
            "fail-array-row-env-write",
            "fail-array-row-conditional-write",
            "fail-array-row-alias-write-barrier",
            "fail-array-row-call-barrier",
            "fail-array-row-object-write-barrier",
            "fail-array-row-interpolation-effect-barrier",
            "fail-array-row-shadow-scope",
        ],
    );
    push(
        &mut owned,
        rule,
        &config,
        &["pass-array-row-overwritten-member"],
    );
    // Four findings, lines 6, 8, 10, and 14, file src/query.ts, target first.
    push(
        &mut owned,
        rule,
        &config,
        &["fail-array-row-unmatched-overwrites"],
    );
    // Bind values are not identifier sources. Targets:
    // fail-table-with-binds: table;
    // fail-helper-column: entityColumn and table;
    // fail-limit-cursor-bind: name.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-table-with-binds",
            "fail-helper-column",
            "fail-limit-cursor-bind",
        ],
    );
    // A hole glued to an identifier suffix or prefix still occupies the name.
    // fail-glued-identifier-hole reports table at lines 6, 16, and 20.
    push(&mut owned, rule, &config, &["fail-glued-identifier-hole"]);
    // Env and config holes outside identifier positions stay findings.
    // Each call also names table. A plain parameter beside them does not.
    // Imported env config, a namespace member, and a helper return stay.
    // An imported literal export does not.
    push(&mut owned, rule, &config, &["fail-env-in-where"]);
    // No finding: a static call, a dynamic call with no interpolation,
    // calls whose only interpolations or arguments are binds, and a
    // conditional relation name with parameter binds.
    push(&mut owned, rule, &config, &["pass-static-or-bind-only"]);
    // Closed registries, stripped uuid names, and dominating throw guards are empty.
    // fail-guard-outside-branch keeps the query after the branch.
    // fail-uuid-unstripped-or-suffix keeps unstripped, single replace, and a suffix.
    // fail-open-object-member keeps a registry value that reads process.env.
    // fail-nested-map-get keeps a get reached through another object.
    // fail-mutated-registry keeps direct writes and any registry use outside a
    // member read, for-of right-hand side, or spread. A computed call callee
    // is a read only for a string-literal read-only method.
    // fail-shadowed-random-uuid keeps a local or non-crypto randomUUID.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "pass-closed-object-registry",
            "pass-closed-array-registry",
            "pass-closed-frozen-registry",
            "pass-closed-map",
            "pass-uuid-temp-name",
            "pass-set-throw-guard",
            "pass-includes-throw-guard",
            "fail-guard-outside-branch",
            "fail-uuid-unstripped-or-suffix",
            "fail-open-object-member",
            "fail-nested-map-get",
            "fail-mutated-registry",
            "fail-shadowed-random-uuid",
        ],
    );
    // Every hole of a sql-template-strings part in an append chain is a bind
    // (pass-sql-template-append-chain). A raw append argument stays a
    // candidate: fail-sql-template-append-raw reports table only. A parameter
    // or local named like the tag shadows the import: fail-sql-template-shadowed-tag.
    // sql-template-strings holes are binds: pass-sql-template-binds is empty.
    // fail-sql-template-raw keeps tableUnions and batchColumn in a plain string.
    let factory = rules_path(rule, "configs", "factory.yml");
    push(
        &mut owned,
        rule,
        &factory,
        &[
            "pass-sql-template-binds",
            "fail-sql-template-raw",
            "pass-sql-template-append-chain",
            "fail-sql-template-append-raw",
            "fail-sql-template-shadowed-tag",
            "pass-sql-template-function-args",
        ],
    );
    // Function-call arguments in FROM, JOIN, or ANY are values, not relation
    // names. pass-raw-function-args binds them; fail-raw-relation-with-function-join
    // keeps reporting table (lines 4, 8, 12) while the unnest arguments stay quiet.
    push(
        &mut owned,
        rule,
        &factory,
        &[
            "pass-raw-function-args",
            "fail-raw-relation-with-function-join",
        ],
    );
    let assertions = rules_path(rule, "configs", "finite-domain-assertions.yml");
    // The assertion helper is bounded. A regex test and an open set stay findings.
    push(
        &mut owned,
        rule,
        &assertions,
        &[
            "pass-finite-domain-assertion",
            "fail-regex-only-guard",
            "fail-open-set-guard",
            "fail-reassigned-guard",
        ],
    );
    let cases: Vec<(&str, PathBuf, PathBuf)> = owned
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect();
    assert_saved_fixture("postgres_no_unbounded_sql_identifier", &cases);
}

#[test]
fn postgres_low_typed_transports_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-typed-transports",
    );
    assert_saved_fixture(
        "postgres_low_typed_transports",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

#[test]
fn postgres_low_recovery_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-recovery",
    );
    assert_saved_fixture(
        "postgres_low_recovery",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}

#[test]
fn postgres_low_await_using_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "low-await-using",
    );
    assert_saved_fixture(
        "postgres_low_await_using",
        &[
            ("low", root.clone(), root.join(".no-mistakes-low.yml")),
            ("max", root.clone(), root.join(".no-mistakes-max.yml")),
        ],
    );
}
