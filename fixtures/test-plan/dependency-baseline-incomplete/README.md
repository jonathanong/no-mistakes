# Missing dependency baseline

Synthetic pnpm workspace. The committed files are the pre-upgrade baseline.
`changes/` is the dependency-only bump of `fixture-analyzer` (root manifest,
tooling manifest, and both pnpm importer entries). No source file changes.

`app` is a Vitest project whose `setupFiles` value is dynamic, so a changed
manifest under the repo root selects only that project's tests. `tooling` has
a direct importer test and a test that reaches the same package through
`helper.ts`. The plan environment is dependencies-only so sampling cannot
hide a miss. Without `--base`, dependency coverage is incomplete and must not
become a full-suite selection.
