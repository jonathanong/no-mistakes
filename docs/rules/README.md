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

| Rule | Purpose |
| --- | --- |
| [`agents-md-max-size`](agents-md-max-size.md) | Keep agent instruction files small enough for context. |
| [`banned-paths`](banned-paths.md) | Ban tracked files matching configured path globs. |
| [`banned-renamed-files`](banned-renamed-files.md) | Ban legacy filenames that should be renamed. |
| [`config-path-references`](config-path-references.md) | Validate path strings in structured config files. |
| [`doc-consistency`](doc-consistency.md) | Require files, headings, substrings, and banned-substring checks. |
| [`file-extension-policy`](file-extension-policy.md) | Enforce allowed or banned extensions in configured scopes. |
| [`finite-set-consistency`](finite-set-consistency.md) | Compare finite string sets extracted from source and paths. |
| [`github-actions-pinned-hash`](github-actions-pinned-hash.md) | Require every `uses:` step to be pinned to a commit SHA with a version comment. |
| [`forbidden-dependencies`](forbidden-dependencies.md) | Prevent configured files/modules from depending on forbidden targets. |
| [`integration-test-no-mocks`](integration-test-no-mocks.md) | Ban mocking libraries and mock helpers in integration tests. |
| [`lockfile-allowlist`](lockfile-allowlist.md) | Allow only configured package lock files. |
| [`markdown-link-display-text`](markdown-link-display-text.md) | Require markdown link text to match the linked file basename. |
| [`nextjs-no-api-routes`](nextjs-no-api-routes.md) | Ban Next.js API route files. |
| [`nextjs-no-caching`](nextjs-no-caching.md) | Ban Next.js caching features. |
| [`no-empty-or-comments-only-files`](no-empty-or-comments-only-files.md) | Ban empty/comment-only files. |
| [`no-git-identity-mutation`](no-git-identity-mutation.md) | Ban scripts that mutate git identity. |
| [`package-json-registry-only`](package-json-registry-only.md) | Require package registries to match configured policy. |
| [`package-json-workspace-coverage`](package-json-workspace-coverage.md) | Require package directories to be covered by workspace config. |
| [`playwright-coverage`](playwright-coverage.md) | Require Playwright route/selector coverage. |
| [`playwright-prefer-test-id-locators`](playwright-prefer-test-id-locators.md) | Prefer test ID locators when matched app elements expose test IDs. |
| [`playwright-unique-html-ids`](playwright-unique-html-ids.md) | Require unique HTML `id` values in Playwright analysis. |
| [`playwright-unique-test-ids`](playwright-unique-test-ids.md) | Require unique test ID values in Playwright analysis. |
| [`require-files-in-subdirs`](require-files-in-subdirs.md) | Require files under matching subdirectories. |
| [`require-storybook-stories`](require-storybook-stories.md) | Require Storybook coverage for selected components. |
| [`require-test-per-subdir`](require-test-per-subdir.md) | Require tests in each first-level subdirectory. |
| [`required-companion-imports`](required-companion-imports.md) | Require companion files to import their paired source. |
| [`required-doc-section`](required-doc-section.md) | Require a heading in matching documentation files. |
| [`required-local-docs`](required-local-docs.md) | Require local docs beside configured code directories. |
| [`rust-max-lines-per-file`](rust-max-lines-per-file.md) | Cap Rust source/test file length. |
| [`rust-no-inline-allows`](rust-no-inline-allows.md) | Ban inline Rust `allow` attributes. |
| [`rust-no-inline-tests`](rust-no-inline-tests.md) | Ban inline Rust test modules. |
| [`server-route-client-boundary`](server-route-client-boundary.md) | Keep generated/direct clients out of server route folders. |
| [`shellcheck-runner`](shellcheck-runner.md) | Run ShellCheck for shell files/scripts. |
| [`strict-package-layout`](strict-package-layout.md) | Enforce configured package file layout. |
| [`structured-config-policy`](structured-config-policy.md) | Require or ban structured config keys. |
| [`test-email-domain-policy`](test-email-domain-policy.md) | Ban configured email domains in tracked fixtures and docs. |
| [`test-no-unmocked-dynamic-imports`](test-no-unmocked-dynamic-imports.md) | Require dynamic imports in tests to be mocked. |
| [`tsconfig-alias-folder-mapping`](tsconfig-alias-folder-mapping.md) | Enforce alias/folder consistency. |
| [`unique-exports`](unique-exports.md) | Prevent ambiguous duplicate public export names. |
| [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) | Require Vitest inputs to be covered by CI path filters. |
| [`vitest-project-mapping`](vitest-project-mapping.md) | Require Vitest tests to map to exactly one project. |
| [`vitest-test-correspondence`](vitest-test-correspondence.md) | Enforce source/test correspondence for Vitest. |
| [`workspace-package-cycles`](workspace-package-cycles.md) | Prevent dependency cycles between workspace packages. |

## Suppression

Use `no-mistakes` directives, not legacy `guardrails` directives:

```ts
// no-mistakes-disable-next-line unique-exports: intentional public alias
export { handler as GET };
```

Top-of-file opt-outs use `no-mistakes-disable-file`. Line suppressions require
rules to report line numbers.
