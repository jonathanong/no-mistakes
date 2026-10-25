# Configuration

`no-mistakes` uses `.no-mistakes.yaml`, `.no-mistakes.yml`,
`.no-mistakes.json`, or `.no-mistakes.jsonc`.

```yaml
projects:
  web:
    type: nextjs
    root: web
tests:
  playwright:
    configs: web/playwright.config.ts
queues:
  factories: [createQueue]
effects:
  valkey:
    categories:
      cache: [ValkeyCache, getEntityCache]
      pubsub: [createPublisher]
effort: max
unanalyzable: report
rules:
  - rule: unique-exports
    projects: [web]
```

The `effects` map declares named effect families for the
[`effects`](../cli/effects.md) query (`effects <kind> --entry <file>`); each
`<kind>` maps category labels to the function/constructor names that belong to
them.

`effort` sets the repository default for rules that support effort selection;
it defaults to `max`, which preserves their current analysis. A rule's
`options.effort` overrides the repository value, and a check invocation's
`effort` overrides both. `unanalyzable` sets the repository policy for inputs a
rule cannot analyze (`report`, `advise`, or `ignore`); a rule's
`options.unanalyzable` overrides it. Any explicit `unanalyzable` or
`unanalyzableSql` value, on the repository or on a rule, replaces the schema
default with that policy. The explicit `report` in this example therefore makes
unanalyzable PostgreSQL body statements fail, and it turns complete
recognized-unsupported top-level forms into findings. An explicit `advise`
still advises those statements, and `ignore` omits them. Omitting both keys
preserves the [schema defaults](../migrations/postgres-schema-unanalyzable.md):
those body statements and recognized-unsupported forms advise, while malformed
or unrecognized top-level statements still fail. The top-level and per-rule
`unanalyzableSql` keys are aliases; legacy PostgreSQL rule parsers may retain
their `fail` spelling, which maps to the shared `report` policy. The shared
`report` value is translated to each legacy parser's expected spelling. At a
repository scope, and for rule applications that consume the shared policy,
canonical `unanalyzable` takes precedence when both keys are set. The alias is
still validated in those cases, so an invalid value remains a configuration
error even when the canonical value is valid. Legacy PostgreSQL applications
that historically ignored this per-rule payload continue to ignore it for
compatibility.
Rules that do not implement low effort continue to use their maximum analysis.
When a low-effort rule requires a partner check that is missing or outside its
configured scope, `check` warns and runs that rule at `max`.
For example, [`github-actions-job-timeouts`](../rules/github-actions-job-timeouts.md#effort)
uses a covering `github-actions-step-timeouts` partner to avoid redundant
step comparisons when the partner caps steps below the job deadline.

The top-level `frontendRoot`, `assertNoFetch`, and `reactTraits` fields are
legacy compatibility settings for the standalone `react` command. Prefer
project-scoped roots and `tests.playwright` settings for new configurations;
the legacy fields do not change shared frontend-app resolution.

## Configuration diagnostics

Invalid-glob configuration errors and `no-mistakes-config` findings include
the selected configuration filename and source line when the parsed value has
an authoritative position. This works for `.yaml`, `.yml`, `.json`, and
`.jsonc` configuration files, including a custom filename selected with
`--config`. An invalid glob remains a fatal configuration error, now
attributed to its entry:

```yaml
# .no-mistakes.yml
rules:
  - rule: some-rule
    exclude: ["src/**", "[bad"]
```

The fatal diagnostic includes the source file, line, rule application, field,
and parser detail:

```text
.no-mistakes.yml:4: rules[0] (some-rule).exclude[1]: contains invalid glob `[bad`: error parsing glob '[bad': unclosed character class; missing ']'
```

A valid glob that matches no files and a missing configured path are rule
findings, with the line of each offending entry; see
[`no-mistakes-config`](../rules/no-mistakes-config.md).

Values supplied programmatically or derived without an authoritative source
position use line 1 rather than an inferred line.

## Topics

- [Discovery](discovery.md)
- [Projects](projects.md)
- [Tests and selectors](tests.md)
- [Queues](queues.md)
- [Effects](effects.md)
- [Rules](rules.md)
- [Test plan](test-plan.md)
- [Filesystem](filesystem.md)
- [CI workflow analysis](ci.md)
- [Workflow secret inventory](workflow-topology-policy.md)
- [Workflow topology concurrency](workflow-topology-policy-concurrency.md)
- [Changed-file checks](checks.md)
- [Infrastructure (Terraform/OpenTofu)](infra.md)
