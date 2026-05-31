# `tsconfig-alias-folder-mapping`

Enforces consistency between TypeScript path aliases and target folders.

```yaml
rules:
  - rule: tsconfig-alias-folder-mapping
    scope: repository
    options:
      tsconfig: tsconfig.json
      baseDir: src
      mappings:
        - prefix: "@api"
          root: api
```

Counterexample: `@api/*` pointing to a folder that does not exist or does not
match the alias prefix policy.

Fix: update `compilerOptions.paths` or rename folders to match.
