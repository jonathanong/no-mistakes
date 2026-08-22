# no-mistakes Rules

These are configured `no-mistakes check` rules. Add them under `rules:` in
`.no-mistakes.yml`.

```yaml
rules:
  - rule: unique-exports
    projects: [web]
```

Counterexample:

```yaml
# Does nothing useful because the rule has no effective target.
rules:
  - rule: unique-exports
```

## Rule Index

| Rule                                                                          | Purpose                                                                         |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| [`agents-md-max-size`](agents-md-max-size.md)                                 | Keep agent instruction files small enough for context.                          |
| [`banned-paths`](banned-paths.md)                                             | Ban tracked files matching configured path globs.                               |
| [`banned-renamed-files`](banned-renamed-files.md)                             | Ban legacy filenames that should be renamed.                                    |
| [`config-path-references`](config-path-references.md)                         | Validate path strings in structured config files.                               |
| [`csharp-max-lines-per-file`](csharp-max-lines-per-file.md)                   | Cap C# source/test file length by physical lines.                               |
| [`doc-consistency`](doc-consistency.md)                                       | Require files, headings, substrings, and banned-substring checks.               |
| [`file-extension-policy`](file-extension-policy.md)                           | Enforce allowed or banned extensions in configured scopes.                      |
| [`finite-set-consistency`](finite-set-consistency.md)                         | Compare finite string sets extracted from source and paths.                     |
| [`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)     | Require paired step and nested action timeouts for configured `uses`.       |
| [`github-actions-composite-step-schema`](github-actions-composite-step-schema.md) | Validate composite-action steps against GitHub's documented step keys.     |
| [`github-actions-job-timeouts`](github-actions-job-timeouts.md)                   | Require literal job `timeout-minutes` and optional caps.                    |
| [`github-actions-pinned-hash`](github-actions-pinned-hash.md)                 | Require every `uses:` step to be pinned to a commit SHA with a version comment. |
| [`github-actions-test-timeout-literals`](github-actions-test-timeout-literals.md) | Reject timeout-minutes literals restated in workflow tests.                 |
| [`forbidden-dependencies`](forbidden-dependencies.md)                         | Prevent configured files/modules from depending on forbidden targets.           |
| [`forbidden-workspace-closure`](forbidden-workspace-closure.md)               | Prevent workspace package closures from reaching forbidden packages.            |
| [`integration-test-no-mocks`](integration-test-no-mocks.md)                   | Ban mocking libraries and mock helpers in integration tests.                    |
| [`lockfile-allowlist`](lockfile-allowlist.md)                                 | Allow only configured package lock files.                                       |
| [`markdown-child-links`](markdown-child-links.md)                                 | Require parent Markdown files to link every matching child.                 |
| [`markdown-eval-tests`](markdown-eval-tests.md)                                   | Ban tests that eval markdown shell blocks unless exact-path allowlisted.    |
| [`markdown-link-display-text`](markdown-link-display-text.md)                 | Require Markdown link text to match the linked file basename.                   |
| [`markdown-mermaid-validation`](markdown-mermaid-validation.md)               | Validate Mermaid diagrams embedded in Markdown fences.                          |
| [`markdown-reachability`](markdown-reachability.md)                           | Require Markdown docs to be reachable from instruction roots.                   |
| [`markdown-structure-budget`](markdown-structure-budget.md)                   | Limit tables and Mermaid diagrams in oversized Markdown.                        |
| [`nextjs-no-api-routes`](nextjs-no-api-routes.md)                             | Ban Next.js API route files.                                                    |
| [`nextjs-no-caching`](nextjs-no-caching.md)                                   | Ban Next.js caching features.                                                   |
| [`nextjs-redirect-destinations`](nextjs-redirect-destinations.md)             | Require Next.js redirect/rewrite destinations to match App Router pages.        |
| [`no-empty-or-comments-only-files`](no-empty-or-comments-only-files.md)       | Ban empty/comment-only files.                                                   |
| [`no-git-identity-mutation`](no-git-identity-mutation.md)                     | Ban scripts that mutate git identity.                                           |
| [`no-mistakes-config`](no-mistakes-config.md)                                 | Lint `.no-mistakes.yml` paths, empty globs, and env-level `limit` with `direct`. |
| [`no-raw-ephemeral-port`](no-raw-ephemeral-port.md)                           | Ban raw ephemeral port 0 binds and Node `listen(0)` calls.                      |
| [`package-json-registry-only`](package-json-registry-only.md)                 | Require package registries to match configured policy.                          |
| [`package-json-workspace-coverage`](package-json-workspace-coverage.md)       | Require package directories to be covered by workspace config.                  |
| [`postgres-constraint-validate`](postgres-constraint-validate.md)                 | Pair named NOT VALID constraint adds with VALIDATE CONSTRAINT.              |
| [`postgres-fk-index`](postgres-fk-index.md)                                       | Require a leading btree/hash index on each foreign key column.              |
| [`postgres-redundant-index`](postgres-redundant-index.md)                         | Flag btree indexes whose keys are a strict prefix of another live index.    |
| [`postgres-no-generated-column-writes`](postgres-no-generated-column-writes.md) | Ban DML writes to PostgreSQL generated columns.                               |
| [`playwright-coverage`](playwright-coverage.md)                               | Require Playwright route/selector coverage.                                     |
| [`playwright-prefer-test-id-locators`](playwright-prefer-test-id-locators.md) | Prefer test ID locators when matched app elements expose test IDs.              |
| [`playwright-unique-html-ids`](playwright-unique-html-ids.md)                 | Require unique HTML `id` values in Playwright analysis.                         |
| [`playwright-unique-test-ids`](playwright-unique-test-ids.md)                 | Require unique test ID values in Playwright analysis.                           |
| [`postgres-lock-ordering`](postgres-lock-ordering.md)                         | Require ORDER BY or SKIP LOCKED on multi-row FOR UPDATE locks.                  |
| [`production-dependency-declarations`](production-dependency-declarations.md) | Require production-reachable imports to be declared as runtime dependencies.    |
| [`require-files-in-subdirs`](require-files-in-subdirs.md)                     | Require files under matching subdirectories.                                    |
| [`require-storybook-stories`](require-storybook-stories.md)                   | Require Storybook coverage for selected components.                             |
| [`require-test-per-subdir`](require-test-per-subdir.md)                       | Require tests in each first-level subdirectory.                                 |
| [`required-companion-imports`](required-companion-imports.md)                 | Require companion files to import their paired source.                          |
| [`required-entrypoint-reachability`](required-entrypoint-reachability.md)     | Require selected sources to be runtime-reachable from configured entrypoints.   |
| [`required-doc-section`](required-doc-section.md)                             | Require a heading in matching documentation files.                              |
| [`required-local-docs`](required-local-docs.md)                               | Require local docs beside configured code directories.                          |
| [`rust-max-lines-per-file`](rust-max-lines-per-file.md)                       | Cap Rust source/test file length.                                               |
| [`rust-no-inline-allows`](rust-no-inline-allows.md)                           | Ban inline Rust `allow` attributes.                                             |
| [`rust-no-inline-tests`](rust-no-inline-tests.md)                             | Ban inline Rust test modules.                                                   |
| [`server-route-client-boundary`](server-route-client-boundary.md)             | Keep generated/direct clients out of server route folders.                      |
| [`shellcheck-runner`](shellcheck-runner.md)                                   | Run ShellCheck for shell files/scripts.                                         |
| [`strict-package-layout`](strict-package-layout.md)                           | Enforce configured package file layout.                                         |
| [`structured-config-policy`](structured-config-policy.md)                     | Require or ban structured config keys.                                          |
| [`test-email-domain-policy`](test-email-domain-policy.md)                     | Ban configured email domains in tracked fixtures and docs.                      |
| [`test-no-dependency-pins`](test-no-dependency-pins.md)                       | Ban exact dependency-version assertions in tests.                               |
| [`test-no-unmocked-dynamic-imports`](test-no-unmocked-dynamic-imports.md)     | Require dynamic imports in tests to be mocked.                                  |
| [`tsconfig-alias-folder-mapping`](tsconfig-alias-folder-mapping.md)           | Enforce alias/folder consistency.                                               |
| [`tsconfig-file-coverage`](tsconfig-file-coverage.md)                         | Require tracked TypeScript files to belong to a tsconfig program.               |
| [`tsconfig-gate-coverage`](tsconfig-gate-coverage.md)                         | Require tracked TypeScript projects to have CI and local typecheck registrations. |
| [`unique-exports`](unique-exports.md)                                         | Prevent ambiguous duplicate public export names.                                |
| [`version-pin-consistency`](version-pin-consistency.md)                       | Keep a structured source pin in lockstep with other files.                      |
| [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md)                       | Require Vitest inputs to be covered by CI path filters.                         |
| [`vitest-project-mapping`](vitest-project-mapping.md)                         | Require Vitest tests to map to exactly one project.                             |
| [`vitest-test-correspondence`](vitest-test-correspondence.md)                 | Enforce source/test correspondence for Vitest.                                  |
| [`workflow-topology-policy`](workflow-topology-policy.md)                     | Declarative GitHub Actions topology inventory, edges, and step order.           |
| [`workspace-package-cycles`](workspace-package-cycles.md)                     | Prevent dependency cycles between workspace packages.                           |

## Suppression

Use `no-mistakes` directives, not legacy `guardrails` directives:

```ts
// no-mistakes-disable-next-line unique-exports: intentional public alias
export { handler as GET };
```

Top-of-file opt-outs use `no-mistakes-disable-file`. Line suppressions require
rules to report line numbers. `no-mistakes check --include-suppressed` is an
opt-in audit view: it adds a deterministic `suppressed` array containing the
domain, rule ID, finding location/reason, and matching directive kind/line.
Unknown rule IDs, malformed directives, and unused directives are ignored;
they never hide a finding. File directives apply even when a finding has no
line, while line and next-line directives require an exact finding location.
