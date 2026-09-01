# `test-no-dependency-pins`

Forbids exact dependency-version assertions in tests. Tests that pin a
package-manifest dependency, parsed dependency-map value, GitHub Action ref,
tool version, release URL, release asset, or tool log line break when the real
pin is bumped and hide the fact that the assertion is too specific.

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

Counterexample: a test asserts a concrete dependency entry read from
`package.json`, compares a `dependencies` or `devDependencies` property to a
concrete version, or asserts `uses: actions/checkout@v6.0.2`,
`NODE_VERSION: '20.11.0'`, `releases/download/v1.2.3`,
`no-mistakes-v0.44.0-x86_64-apple-darwin.tar.gz`, or `RUN v1.2.3`.

```ts
expect(readFileSync('package.json', 'utf8')).toContain('"no-mistakes": "0.53.2"')
expect(packageJson.devDependencies?.['no-mistakes']).toBe('^0.53.2')
expect(workflow).toContain('uses: actions/checkout@v6.0.2')
expect(env).toContain("NODE_VERSION: '20.11.0'")
expect(script).toContain('releases/download/v1.2.3')
```

Fix: assert a stable invariant instead of a concrete version — for example
check that the dependency exists, verify that related dependency ranges agree,
match `actions/checkout@` without the tag, or compare a generated fixture.
Keep exact versions in production installers and workflow files; this rule
only scans test files.

Use `no-mistakes-disable-next-line test-no-dependency-pins` or
`no-mistakes-disable-line` for a one-off fixture, or
`no-mistakes-disable-file` when a whole test file is an intentional snapshot
of pinned versions.

## Why and when

Use this rule when tests assert generated workflows, installers, or release
metadata that changes independently of the behavior under test.

## What it catches/requires

Selected test files must not assert exact package-manifest dependency,
dependency-map, action, tool, release URL, asset, or log version strings matched
by the configured pin patterns. Manifest matching is deliberately limited to
same-line `readFileSync('package.json', ...)` or `readRepoFile('package.json')`
assertions containing a quoted entry with a bare version, caret range, or tilde
range; fixture paths and entries named `version` are not matched. Because
raw text assertions do not preserve the entry's manifest section, suppress an
intentional metadata version assertion locally. Raw reads may use `toString()`
or `trim()` before `toContain`, `toBe`, `toEqual`, or `toStrictEqual`. Parsed matching is limited to
`dependencies`, `devDependencies`, `optionalDependencies`, and
`peerDependencies` property or bracket access followed directly by `toBe`,
`toEqual`, or `toStrictEqual`, including computed identifier or member keys,
or dependency-valued `toHaveProperty` assertions. It supports non-null
assertions, multiline assertions (including `expect.soft` and `expect.poll`),
simple identifier or member-path `as` casts, extra parenthesized wrappers,
optional string-literal assertion messages, explicit equality, compound
comparator, OR, or hyphen ranges, and versioned `npm:` or `workspace:` specs.
Computed dependency keys are limited to identifiers and member paths; calls,
concatenations, nested bracket expressions, and array-form property paths are
not matched. Negated assertions and malformed version prefixes are not matched.

## Options and defaults

There is no user-facing `defaultInclude` option. Internally, when `include` is
omitted, the default include is Filaments `TEST_FILE_RE`: `__tests__/` and
`*.test.{mts,ts,tsx,mjs,js,cts,cjs}`, including mocks. `include` replaces that
set; `patterns` replaces the default pin regexes. Both options default to the
shown behavior when omitted.

## Valid example

```ts
expect(workflow).toContain("uses: actions/checkout@");
```

## Counterexample

```ts
expect(workflow).toContain("uses: actions/checkout@v6.0.2");
```

## Fix

Assert the stable prefix or behavior, compare against the current source pin,
or move the exact version assertion into a focused production-release check.

## Suppression

Use `no-mistakes-disable-next-line test-no-dependency-pins` for a single fixture
exception, or the file directive for a test intentionally snapshotting pins.

## Related rules

[`version-pin-consistency`](version-pin-consistency.md) checks that a source pin
and its anchors agree; [`test-email-domain-policy`](test-email-domain-policy.md)
keeps test fixtures synthetic.
