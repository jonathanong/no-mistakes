fn extend_identifier_cases_1(
    owned: &mut Vec<(String, PathBuf, PathBuf)>,
    rule: &str,
    configs: &[PathBuf; 3],
) {
    // Fixture configs that only differ by effort are not these cases.
    // A finding file ends with query.ts and the message contains
    // "finite-domain validator".
    push(
        owned,
        rule,
        &configs[0],
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
        owned,
        rule,
        &configs[0],
        &[
            "pass-switch",
            "pass-hasown-record",
            "pass-computed-member-return",
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
        owned,
        rule,
        &configs[0],
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
        owned,
        rule,
        &configs[0],
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
        owned,
        rule,
        &configs[0],
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
        owned,
        rule,
        &configs[0],
        &["pass-array-row-overwritten-member"],
    );
    // Four findings, lines 6, 8, 10, and 14, file src/query.ts, target first.
    push(
        owned,
        rule,
        &configs[0],
        &["fail-array-row-unmatched-overwrites"],
    );
    // Bind values are not identifier sources. Targets:
    // fail-table-with-binds: table;
    // fail-helper-column: entityColumn and table;
    // fail-limit-cursor-bind: name.
    push(
        owned,
        rule,
        &configs[0],
        &[
            "fail-table-with-binds",
            "fail-helper-column",
            "fail-limit-cursor-bind",
        ],
    );
}
