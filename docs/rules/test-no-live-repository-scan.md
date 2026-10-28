# `test-no-live-repository-scan`

Rejects directory enumeration of the live repository that a Vitest root runs.
A test that lists `.github/workflows` at module load and then reads and parses
every file it found checks the repository, not the code under test. The result
changes whenever an unrelated file is added, and the work repeats on every test
run.

```yaml
rules:
  - rule: test-no-live-repository-scan
    scope: repository
    severity: advisory
    options:
      roots:
        - file: tests/policy.test.ts
```

Without `roots` the rule starts from every Vitest project (`tests.vitest`);
this example names one file. `severity: advisory` is the recommended adoption
default: findings are listed without failing `check`. Remove it once the
repository is clean.

## Why and when

Use this rule when tests read policy from the repository itself (workflow
policy, lint-config shape, file-layout invariants). Run that check once in a
dedicated static CI command and keep the Vitest test for bounded fixtures that
exercise the evaluator. The rule does not ban filesystem access: reading a
fixture, or a directory the test created, is fine.

## What it catches

Starting from the configured roots (every Vitest project by default), the rule
follows the canonical call graph: the root files' own calls, their module scope,
every function they reach, and the module scope of every module they import.
At each reached call it reports a directory enumeration whose root is not
bounded:

- `readdir`, `readdirSync`, `opendir`, `opendirSync`, `glob`, and `globSync` from
  `fs` and `fs/promises` (with or without the `node:` prefix), including
  `fs.promises.readdir`.
- Any call into the glob packages `glob`, `fast-glob`, `tinyglobby`, and
  `globby`, other than their pattern helpers (`escapePath`, `isDynamicPattern`,
  and similar).
- Renamed imports (`import { readdirSync as ls }`), default imports, namespace
  imports, an imported helper module, and a wrapper around that helper all
  resolve through the same graph, so the finding is reported at the enumeration
  call wherever it lives.

The enumeration is the trigger, not the later `readFileSync`: a scan that is
never read is still a scan, and every file it yields is covered by the finding
on the enumeration. A `readFileSync` of a fixed literal path is not reported.

## Allowed input

An enumeration passes when its root is provably bounded:

- Any path segment equals a `fixtureDirs` entry (`fixtures` or `__fixtures__` by
  default), such as `join(import.meta.dirname, "fixtures", name)`.
- The root is anchored at `os.tmpdir()` or at the result of `mkdtemp`/
  `mkdtempSync`, held directly or through a `const`.

## Unresolved roots

When the root's provenance cannot be established, the rule reports it with the
reason `unresolved root` instead of passing silently. This covers a function
parameter, a `let` binding, an environment value, a call this analysis does not
model, a spread argument, and an enumeration call whose receiver is not
resolved. These go to the non-failing `advisories` list by default. Set
`unresolvedRoot: report` to fail on them.

Provenance is syntactic. The rule follows `join`, `resolve`, `normalize`,
`dirname`, `fileURLToPath`, `new URL(..., import.meta.url)`, `import.meta.dirname`,
`__dirname`, `process.cwd()`, `tmpdir()`, `mkdtemp`, template literals, and
immutable `const` aliases. A generic helper such as
`listFiles(dir: string)` is therefore an unresolved root at its enumeration even
when every caller passes a fixture; pass a `const` fixture path at the call site
or suppress the helper.

## Options

```yaml
rules:
  - rule: test-no-live-repository-scan
    scope: repository
    options:
      roots:
        - vitest: [unit]
        - glob: scripts/**/*.test.mts
      excludeRoots: ["**/*.mock.test.*"]
      fixtureDirs: [fixtures, __fixtures__, golden]
      enumerators:
        - module: node:fs
          export: cpSync
      unresolvedRoot: advise
```

- `roots` uses the [`forbidden-calls` root selectors](forbidden-calls.md#options-and-roots)
  (`vitest`, `glob`, `file`, `module`, `function`). Default: `[{ vitest: true }]`,
  which needs `tests.vitest` configuration.
- `excludeRoots` removes matching files from the root set, as in `forbidden-calls`.
- `fixtureDirs` lists path segments that mark bounded fixture input. It replaces
  the defaults. Entries are single segments without separators.
- `enumerators` adds enumerating imports to the built-in set. `export` is
  optional; omit it to match every export of `module`.
- `unresolvedRoot` is `advise` (default) or `report`.
- The rule entry's `message` replaces the default finding text.

The common `include` and `exclude` filters narrow which files may carry a
finding; they do not remove traversal roots.

## Valid example

```ts
// tests/policy.test.ts
import { mkdtempSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const root = mkdtempSync(join(tmpdir(), "policy-"));
writeFileSync(join(root, "ci.yml"), "name: ci\n");
const names = readdirSync(root);
const bounded = readdirSync(join(import.meta.dirname, "fixtures", "workflows"));
```

## Counterexample

```ts
// tests/policy.test.ts
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const workflows = join(import.meta.dirname, "..", ".github", "workflows");
const entries = readdirSync(workflows).map((name) =>
  readFileSync(join(workflows, name), "utf8"),
);
```

Reported at the `readdirSync` call: it enumerates a path under the test
module's directory and a Vitest root reaches it.

## Fix

Move the repository-wide check into a released command that runs once in static
CI, and keep the Vitest test for small supplied fixtures that exercise that
check's evaluator. If a test truly needs a directory tree, create it under
`mkdtemp(join(tmpdir(), ...))` or commit it under a `fixtures` directory.

## Suppression

Use `no-mistakes-disable-next-line test-no-live-repository-scan` or
`no-mistakes-disable-line test-no-live-repository-scan` on the enumeration, or
`no-mistakes-disable-file test-no-live-repository-scan` for a whole file, with a
reason. Suppressions are reported by
[suppression hygiene](../configuration/suppressions.md) when they match nothing.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

`options.unanalyzable` is accepted but not used; `unresolvedRoot` controls how
unresolved roots are reported, and they are never omitted.

## Related rules

- [`forbidden-calls`](forbidden-calls.md) bans a configured call from the same
  roots; this rule adds path provenance so bounded fixture I/O stays allowed.
- [`test-no-unmocked-dynamic-imports`](test-no-unmocked-dynamic-imports.md) and
  [`integration-test-no-mocks`](integration-test-no-mocks.md) also select test
  roots through the Vitest catalog.
