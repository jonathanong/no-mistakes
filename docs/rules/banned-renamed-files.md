# `banned-renamed-files`

Bans configured legacy filenames and reports the replacement name.

```yaml
rules:
  - rule: banned-renamed-files
    scope: repository
    options:
      scope: web
      bannedBasenames:
        - name: middleware
          message: "rename middleware.{ts,mts,js} to proxy.ts"
      extensions: [".ts", ".mts", ".js"]
```

By default a basename matches only `<name>.<ext>` where `<ext>` is one of
`extensions`. The last dot is the extension boundary, so `middleware.ts` matches
`middleware` but `middleware.config.ts` does not.

## Multi-part suffixes (`matchCompoundExtensions`)

Set `matchCompoundExtensions: true` on a basename to also match
`<name>.<anything>.<ext>` — i.e. one or more extra segments before a final
extension that is still in `extensions`. This catches every compound variant
without enumerating them:

```yaml
rules:
  - rule: banned-renamed-files
    scope: repository
    options:
      scope: web
      bannedBasenames:
        - name: webpack.config
          message: "remove webpack.config.* — bundling is handled by the framework"
          matchCompoundExtensions: true
      extensions: [".ts", ".js"]
```

This flags `webpack.config.js`, `webpack.config.prod.js`, and
`webpack.config.dev.ts`. It does **not** flag `webpack.configuration.js`: the
match requires the banned name followed by a dot, so an unrelated longer
basename is left alone. The final extension must still be in `extensions`, so
`webpack.config.prod.py` is ignored.

## Regex patterns (`pattern`)

For full control, give a basename a `pattern` (a regex matched against the file
basename). A `pattern` is authoritative: the `extensions` filter is **not**
applied to entries that set it.

```yaml
rules:
  - rule: banned-renamed-files
    scope: repository
    options:
      scope: web
      bannedBasenames:
        - name: jest.config
          message: "remove jest.config.* — tests run under vitest"
          pattern: "^jest\\.config\\..+"
      extensions: [".ts", ".mts", ".js"]
```

This flags `jest.config.js`, `jest.config.cjs`, and `jest.config.mjs`
regardless of `extensions`, while `vitest.config.ts` is untouched. An invalid
regex is a configuration error and fails the run with a message naming the bad
`pattern`.

Counterexample: keeping both old and new filenames during a migration, or a
distinct basename that merely shares a prefix (e.g. `webpack.configuration.js`
under a `matchCompoundExtensions` rule) — neither is flagged.

Fix: rename the file and update imports or framework references.

Suppression caveat: findings are reported at line 1 of the offending file, so
the practical opt-out is a top-of-file `no-mistakes-disable-file
banned-renamed-files` directive. A line-specific
`no-mistakes-disable-next-line` is awkward for a whole-file rename finding.
