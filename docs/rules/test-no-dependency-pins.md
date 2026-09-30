# `test-no-dependency-pins`

Forbids exact dependency-version assertions in tests. Tests that pin a
package-manifest dependency, parsed dependency-map value, GitHub Action ref,
tool version, release URL, release asset, tool log line, container image tag or
digest, setup-action runtime version, versioned Homebrew formula, or versioned
runner label break when the real pin is bumped and hide the fact that the
assertion is too specific.

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
`no-mistakes-v0.44.0-x86_64-apple-darwin.tar.gz`, `RUN v1.2.3`,
`image: redis:7.4-alpine`, `node-version: '26'`, `brew install postgresql@18`,
or `runs-on: ubuntu-24.04-arm`.

```ts
expect(readFileSync('package.json', 'utf8')).toContain('"no-mistakes": "0.53.2"')
expect(packageJson.devDependencies?.['no-mistakes']).toBe('^0.53.2')
expect(workflow).toContain('uses: actions/checkout@v6.0.2')
expect(env).toContain("NODE_VERSION: '20.11.0'")
expect(script).toContain('releases/download/v1.2.3')
expect(compose).toContain('image: redis:7.4-alpine')
expect(workflow).toContain("node-version: '26'")
expect(script).toContain('brew install postgresql@18')
expect(workflow).toContain('runs-on: ubuntu-24.04-arm')
```

Fix: assert a stable invariant instead of a concrete version — for example
check that the dependency exists, verify that related dependency ranges agree,
match `actions/checkout@` without the tag, read the image, runtime, or runner
from the workflow under test instead of restating it, or compare a generated
fixture.
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
`peerDependencies` property or bracket access followed, aside from JavaScript
trivia, by `toBe`, `toEqual`, or `toStrictEqual`, including computed identifier or member keys,
or dependency-valued `toHaveProperty` assertions. It supports non-null
assertions, multiline assertions (including `expect.soft` and `expect.poll`),
the transparent `resolves` modifier,
sync or async expression callbacks and single-return block callbacks passed to `expect.poll`,
simple identifier or member-path `as` casts, extra parenthesized wrappers,
optional string-literal assertion messages, explicit equality, compound
comparator, OR, or hyphen ranges, and versioned `npm:` or `workspace:` specs.
Parsed object, property, and computed-key identifiers support ECMAScript Unicode identifiers.
Computed dependency keys are limited to identifiers and member paths; calls,
concatenations, nested bracket expressions, array-form property paths, and
computed expected-version strings are not matched. Negated assertions and
malformed version prefixes are not matched.

## Container images, setup versions, formulae, and runners

Five default pin kinds cover pins that live in workflow, Compose, Dockerfile, and
install-script text rather than in a manifest. Each runs line by line, reports
only the pin (not the surrounding line), and reports several pins per line.

- `container image tag`: `owner/name:9.1.0`, `host.io/name:2.4`, `name:24-slim`,
  `owner/name:pg18`, and `image: owner/name:9`, with or without a trailing
  `@sha256:` digest of 64 hex characters. A tag followed by an interpolated or
  short digest (`owner/name:9.1.0@sha256:${digest}`) reports only the tag. Not
  `name:latest`, untagged images, `${tag}` interpolation, or ports.
- `container image digest`: an untagged `name@sha256:` plus 64 hex characters.
  Not short or repeated digests, or `${digest}` interpolation.
- `setup action version`: `node-version: 26`, `python-version: "3.13"`, and the
  `go`, `java`, `ruby`, `dotnet`, `php`, `bun`, and `deno` forms. Not
  `node-version-file:`, `lts/*`, or `${{ matrix.node }}`.
- `versioned Homebrew formula`: `postgresql@18` or `openssl@3.5` on a line that
  mentions `brew`, `homebrew`, `linuxbrew`, or `/Cellar/`. Not `pnpm@12` or
  `undici@1.0.1` without that context.
- `versioned runner label`: `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15`,
  `macos-15-intel`, `windows-2025`, and `windows-11-arm`. Not `ubuntu-latest`,
  `ubuntu-slim`, `macos-latest`, or `windows-1252`.

An image is recognized by its grammar, not by a word list. A path under a
dotted registry host with a version-shaped or bare integer tag, or an
`owner/name` path with a version-shaped tag (`1.2.3`, `v1.2.3`, `24-slim`,
`pg18`), is an image anywhere on the line. A bare integer tag on an `owner/name`
path
(`owner/name:9`) and a `name:tag` with no slash are only images after `image:` or
`FROM`, because `file.mts:12`, `dev/initialize:2`, and Go or Node stack frames
have the same shape. Repository components never contain dots, which keeps
`binder.go:1755` out, and `-p 51088:6379` port mappings never match.

### Placeholder values

A pin made only of zero versions or of reserved test values is a placeholder,
not a dependency, so it is not reported. This lets a test use a clearly fake
value without a suppression comment:

- every dotted version in the value is all zeros:
  `lychee-v0.0.0-test-x86_64-unknown-linux-gnu.tar.gz`,
  `releases/download/0.0.0-test`;
- an image registry is reserved for testing and documentation:
  `registry.test/app:1.2.3`, `localhost:5000/app:1.2.3`, `example.com/app:1.2.3`,
  and the `.test`, `.example`, `.invalid`, and `.localhost` top-level domains;
- an untagged image digest is one short block repeated to 64 characters, such
  as `sha256:` followed by 64 zeros or `0123456789abcdef` four times.

The exemption is structural: a value is never exempt because it contains the
word `test` or `fake`, and `1.2.3`, `0.0.1`, and a tagged image on a real
registry are still pins. It applies to the default patterns only; custom
`patterns` report every match.

### Precision and recall

Precision comes first because a noisy rule gets suppressed wholesale. The
patterns require a boundary on both sides of the pin, a version-shaped tag, and,
for the ambiguous shapes, a same-line context (`image:`, `FROM`, or Homebrew).

Not caught, by design:

- images or runners built from variables (`${image}`, `${{ matrix.os }}`),
  split across lines (`image:` with the value on the next line), or written as a
  multi-part concatenation;
- an image with no slash, or with a bare integer tag, outside `image:` and
  `FROM` lines, such as `docker run postgres:18`;
- an image written inside a JavaScript regular expression, where the path
  separator is escaped (`/^grafana\/loki:3.4$/`);
- non-version tags such as `latest`, `stable`, or `bookworm`;
- tool versions in log output (`lychee 0.24.2`, `gitleaks version 8.30.1`) and
  release URLs whose tag has a name prefix
  (`releases/download/lychee-v0.24.2/`), which the existing release patterns
  do not match either;
- Homebrew formulae on a line with no Homebrew context, and `@N` package
  specifiers in other package managers (`pnpm@12`, `npm i undici@1`);
- runner labels other than the `ubuntu`, `macos`, and `windows` families;
- ambiguous shapes: `pkg/sub:1.2` reads as an image, because Docker Hub images
  such as `grafana/loki:3.4` have exactly that form.

## Options and defaults

There is no user-facing `defaultInclude` option. Internally, when `include` is
omitted, the default include is Filaments `TEST_FILE_RE`: `__tests__/` and
`*.test.{mts,ts,tsx,mjs,js,cts,cjs}`, including mocks. `include` replaces that
set; `patterns` replaces the default pin regexes, including the container image,
setup version, Homebrew formula, and runner label patterns and the placeholder
exemption. Both options default to the shown behavior when omitted.

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
