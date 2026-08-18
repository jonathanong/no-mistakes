# `test-no-dependency-pins`

Forbids exact dependency-version assertions in tests. Tests that pin a GitHub
Action ref, tool version, release URL, release asset, or tool log line break
when the real pin is bumped and hide the fact that the assertion is too
specific.

```yaml
rules:
  - rule: test-no-dependency-pins
    scope: repository
```

Default include matches Filaments `TEST_FILE_RE`: files under `__tests__/` and
`*.test.{mts,ts,tsx,mjs,js,cts,cjs}` (including `*.mock.test.*`). Override
with `include` globs or replace the default pin regexes with `patterns`.

```yaml
rules:
  - rule: test-no-dependency-pins
    scope: repository
    options:
      include:
        - "**/*.test.ts"
        - "**/__tests__/**"
      patterns:
        - reason: exact action ref
          regex: '(?<!@)\b[\w.-]+/[\w.-]+@(?:v?\d+(?:\.\d+)*|[a-f0-9]{40})(?:\s*#\s*v?\d+(?:\.\d+)*)?\b'
```

Counterexample: a test asserts `uses: actions/checkout@v6.0.2`,
`NODE_VERSION: '20.11.0'`, `releases/download/v1.2.3`,
`no-mistakes-v0.44.0-x86_64-apple-darwin.tar.gz`, or `RUN v1.2.3`.

```ts
expect(workflow).toContain('uses: actions/checkout@v6.0.2')
expect(env).toContain("NODE_VERSION: '20.11.0'")
expect(script).toContain('releases/download/v1.2.3')
```

Fix: assert a looser shape instead of a concrete version — for example match
`actions/checkout@` without the tag, compare a generated fixture, or read the
current pin from the file under test. Keep exact versions in production
installers and workflow files; this rule only scans test files.

Use `no-mistakes-disable-next-line test-no-dependency-pins` or
`no-mistakes-disable-line` for a one-off fixture, or
`no-mistakes-disable-file` when a whole test file is an intentional snapshot
of pinned versions.
