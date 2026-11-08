# Configuration

`no-mistakes` uses `.no-mistakes.yaml`, `.no-mistakes.yml`,
`.no-mistakes.json`, or `.no-mistakes.jsonc`.

For automatic discovery, a repository can also split sections into a
`.no-mistakes/` directory. Each direct child names a top-level section, such as
`.no-mistakes/rules/` or `.no-mistakes/tests/`; files below that directory
contain fragments of that section without repeating its outer key. For
example, a file in `.no-mistakes/rules/` contains a list of rule entries. Any
subdirectories below a section are organizational only. Fragment files accept
the same `.yaml`, `.yml`, `.json`, and `.jsonc` extensions as the root config.

The optional root config is merged first, followed by fragment files in sorted
relative-path order; each file's original list order is retained. Mappings
deep-merge and lists concatenate. Top-level scalar settings stay in the root
file because they have no section directory. Repeating a scalar key or merging
incompatible value types is an error that identifies the source files; values
are never silently overwritten. Duplicate named entries in a list section
across fragment files are errors that identify both files. Existing duplicate
validation for a single config file remains in force. Unknown section names,
unrecognized file extensions (except dotfiles and Markdown files), and config
files placed directly in `.no-mistakes/` are errors. Paths inside fragments,
including project roots and test paths, remain relative to the repository root.
An explicit `--config <file>` loads exactly that file and does not load
`.no-mistakes/` fragments.

When splitting an existing config, keep top-level scalar values inline and move
section contents into matching fragment directories. See the
[split-config migration guide](../migrations/split-no-mistakes-config.md) for
how to update code that reads the config and CI path filters.

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
reportUnusedDisableDirectives: advise
reportUnusedAllowEntries: advise
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

`reportUnusedDisableDirectives` defaults to `advise`: stale disable comments and
comments naming unknown or disabled check rules produce non-failing advice.
Use `report` to fail or `ignore` to omit it. See
[suppression hygiene](suppressions.md) for locations, examples, and API parity.

`reportUnusedAllowEntries` independently controls proven stale configured
exceptions: `advise` (default) returns `unused-allow-entry` advice, `report`
fails the check, and `ignore` omits those diagnostics. Incomplete selector
coverage never establishes staleness, but a literal file path within the configured
scope is also stale when absent from both the visible inventory and disk. See [allow-entry hygiene](suppressions.md#allow-entry-hygiene).

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
- [Splitting a config file](../migrations/split-no-mistakes-config.md)
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
- [Workflow activity type policy](workflow-topology-policy-activity-types.md)
- [Changed-file checks](checks.md)
- [Infrastructure (Terraform/OpenTofu)](infra.md)

Configure opt-in [coverage producer assertion ownership](../rules/coverage-producer-assertion-owner.md) through rule options: exact producer functions, binding-aware assertion imports, explicit ownership contracts, and traversal bounds.

The `postgres-lock-written-rows-only` rule accepts `executorOptionProperties`
(default `[]`) for explicit static transaction-routing options such as
`{ query: tx }`. See the [rule options](../rules/postgres-lock-written-rows-only.md#resource-scoped-transactions-and-executor-options).
