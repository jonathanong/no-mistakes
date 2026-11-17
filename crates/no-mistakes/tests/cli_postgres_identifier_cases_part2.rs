fn extend_identifier_cases_2(
    owned: &mut Vec<(String, PathBuf, PathBuf)>,
    rule: &str,
    configs: &[PathBuf; 3],
) {
    // A hole glued to an identifier suffix or prefix still occupies the name.
    // fail-glued-identifier-hole reports table at lines 6, 16, and 20.
    push(owned, rule, &configs[0], &["fail-glued-identifier-hole"]);
    // Env and config holes outside identifier positions stay findings.
    // Each call also names table. A plain parameter beside them does not.
    // Imported env config, a namespace member, and a helper return stay.
    // An imported literal export does not.
    push(
        owned,
        rule,
        &configs[0],
        &[
            "fail-env-in-where",
            "fail-identifier-with-bind-alias",
            "fail-outside-alias-sources",
        ],
    );
    // No finding: a static call, a dynamic call with no interpolation,
    // calls whose only interpolations or arguments are binds, and a
    // conditional relation name with parameter binds.
    push(
        owned,
        rule,
        &configs[0],
        &["pass-static-or-bind-only", "pass-sql-template-plain-call"],
    );
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
        owned,
        rule,
        &configs[0],
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
    push(
        owned,
        rule,
        &configs[1],
        &[
            "pass-sql-template-binds",
            "fail-sql-template-raw",
            "pass-sql-template-append-chain",
            "fail-sql-template-append-raw",
            "fail-sql-template-shadowed-tag",
            "pass-sql-template-function-args",
            "pass-sql-template-sibling-parameter",
            "pass-sql-template-statement-append",
            "fail-sql-template-shadowed-declarations",
        ],
    );
    // Function-call arguments in FROM, JOIN, or ANY are values, not relation
    // names. pass-raw-function-args binds them; fail-raw-relation-with-function-join
    // keeps reporting table (lines 4, 8, 12) while the unnest arguments stay quiet.
    push(
        owned,
        rule,
        &configs[1],
        &[
            "pass-raw-function-args",
            "fail-raw-relation-with-function-join",
        ],
    );
    // Closed domains. A catalog-typed or numeric parameter, a catalog name
    // plus numeric suffixes, a local alias of either, and map/join over an
    // `as const` list are bounded. fail-open-string-parameter keeps `table`
    // at lines 4 and 8 and the alias `t` at 13; fail-catalog-open-callback
    // keeps refs 6, list 11, and t 19. fail-glued-digit-suffix reports only
    // schema (line 4). Shadowing is scoped to the tag call:
    // fail-sql-template-shadowed-declarations reports table at 8, 13, and 21.
    push(
        owned,
        rule,
        &configs[0],
        &[
            "pass-exported-catalog",
            "mixed-catalog-numeric-suffix",
            "pass-numeric-parameter",
            "mixed-catalog-local-alias",
            "pass-catalog-map-join",
            "pass-user-named-placeholder-table",
            "fail-open-string-parameter",
            "fail-catalog-open-callback",
            "fail-glued-digit-suffix",
        ],
    );
    // The assertion helper is bounded. A regex test and an open set stay findings.
    push(
        owned,
        rule,
        &configs[2],
        &[
            "pass-finite-domain-assertion",
            "fail-regex-only-guard",
            "fail-open-set-guard",
            "fail-reassigned-guard",
        ],
    );
}
