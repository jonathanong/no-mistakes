# Rule Applications

Configured rules are reusable applications, not a single ESLint-style map.

```yaml
rules:
  - name: web-public-api
    rule: unique-exports
    projects: [web]
    include: ["src/**/*.ts"]
    exclude: ["src/generated/**"]
    message: Keep exports unique so agents can trace symbols.
```

Common fields: `name`, `rule`, `message`, `enabled`, `projects`, `tests`,
`scope`, `include`, `exclude`, and rule-specific `options`.

Rules are opt-in. `include` and `exclude` filters apply to each rule
application and are interpreted relative to the configured root.

For [`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md),
`options.unanalyzableSql` defaults to `report`. Configure `ignore` explicitly if
opaque executor arguments are intentional; helper and callback tracing still
checks every recovered leading prefix.

Some rules, including [`forbidden-calls`](../rules/forbidden-calls.md), may be
applied repeatedly. Give overlapping applications names so findings identify the
policy boundary that produced them.

[`declared-payload-compatibility`](../rules/declared-payload-compatibility.md)
accepts `options.contracts`, an opt-in list of named producer/consumer JSON
Schema references. Paths are relative to the invocation root; optional RFC 6901
pointers select each declaration. It checks supported declaration compatibility
and reports unsupported schemas as unproven, without inferring runtime adoption.

For [`github-actions-job-timeouts`](../rules/github-actions-job-timeouts.md),
`options.rejectJobAtOrAboveStepSum` defaults to `false`. Enable it to require a
job timeout below the sum of at least two steps' supported literal timeout
bounds; missing or unsupported step bounds make the sum unprovable and skip
that check. This option is independent of caps and `allow` entries.

For [`github-actions-script-expressions`](../rules/github-actions-script-expressions.md),
`options.constantContexts` defaults to the constant GitHub, runner, and quoted
service-port patterns documented on that page. Omitting the option keeps that
default. An explicit list, including `[]`, replaces it. The rule stays off
until a configuration lists it.
