# `required-companion-imports`

Requires selected source files to have at least one companion file matching a
configured glob, and requires that companion file to import the expected source
specifier.

```yaml
rules:
  - rule: required-companion-imports
    scope: repository
    options:
      sourceDirs: [src/components]
      directChildOnly: true
      sourceExtensions: [.tsx]
      excludeBasenames: [Internal.tsx]
      excludePrefixes: [_]
      companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
      specifierTemplate: "@/components/{sourceStem}"
      stripSourcePrefix: src/
```

Counterexample: `src/components/Button.tsx` exists but
`src/components/Button.stories.tsx` is missing, or the story imports `./Button`
when the configured source specifier is `@/components/Button`.

Fix: add the companion file, update the import to the configured specifier, or
exclude intentional files by basename, prefix, or source selection.
