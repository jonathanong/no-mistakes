# `test-no-live-repository-scan`
<!-- cspell:ignore fdir -->

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
  `fs`, `fs/promises` (with or without the `node:` prefix), and `fs-extra`, including
  `fs.promises.readdir`.
- Any call into the glob packages `glob`, `fast-glob`, `tinyglobby`, and
  `globby`, other than their pattern helpers (`escapePath`, `isDynamicPattern`,
  and similar).
- Renamed imports (`import { readdirSync as ls }`), default imports, namespace
  imports, an imported helper module, and a wrapper around that helper all
  resolve through the same graph, so the finding is reported at the enumeration
  call wherever it lives.
- Direct `import.meta.glob` and `import.meta.globEager` calls, including static
  bracket access such as `import.meta["glob"]`. Literal patterns are relative
  to the module. Dynamic patterns and pattern arrays produce an unresolved-root
  advisory when their boundedness cannot be established.

The built-in matcher does not cover `fdir` builder chains or enumeration through
subprocess commands such as `git ls-files`. These are outside its current scope;
use a dedicated static CI check for them. `enumerators` can add ordinary imported
enumeration functions, but does not interpret subprocess command arguments or
arbitrary builder chains. Aliases of `import.meta.glob` are also outside the
direct-call matcher; keep the API call direct so this rule can inspect it.

The enumeration is the trigger, not the later `readFileSync`: a scan that is
never read is still a scan, and every file it yields is covered by the finding
on the enumeration. A `readFileSync` of a fixed literal path is not reported.

## Allowed input

An enumeration passes when its root is provably bounded:

- Any path segment equals a `fixtureDirs` entry (`fixtures` or `__fixtures__` by
- Any path segment equals a `fixtureDirs` entry (`fixtures` or `__fixtures__` by
  default), such as `join(import.meta.dirname, "fixtures", name)`. This includes
  `.github/fixtures`; the directory name is the boundary, regardless of depth.
  Thus `.github/fixtures/x` is allowed, while `.github/fixtures/../workflows`
  is reported: normalization removes the `fixtures` segment before matching.
- The root is anchored at the result of `mkdtemp`/`mkdtempSync`, held directly,
  through a `const`, or through a `let` whose every write is such a root.
- The root is a parameter, or an environment variable, and the test supplies an
  owned root (see [Owned parameters and env vars](#owned-parameters-and-env-vars)).
- The call forwards a mock's own arguments unchanged
  (see [Pass-through mocks](#pass-through-mocks)).
- The read is under a declared [runtime asset](#runtime-assets); it is listed
  separately instead of failing.

`readdirSync(tmpdir())` and a predictable child such as
`readdirSync(join(tmpdir(), "policy"))` are reported: the shared temporary
directory is not owned by the test. Create a fresh directory with `mkdtemp`
instead. A matching `fixtureDirs` segment remains an explicit allowance.

## Unresolved roots

When the root's provenance cannot be established, the rule reports it with the
reason `unresolved root` instead of passing silently. This covers a function
parameter, a `let` binding, an environment value, a call this analysis does not
model, a spread argument, and an enumeration call whose receiver is not
resolved. These go to the non-failing `advisories` list by default. Set
`unresolvedRoot: report` to fail on them; the rule's `severity: advisory` still
demotes them.

Statically recognized top-level `const` CommonJS bindings participate in path
provenance, including namespace bindings and renamed destructured bindings.
The loader must be the global runtime `require`. Runtime imports named
`require` are separate bindings; erased type-only imports do not shadow the loader.
Nested, mutable, or otherwise unsupported bindings retain the unresolved fallback.
A resolved scan through one binding
does not suppress an unresolved load of the same module through another binding
in that file. Each scan remains a distinct occurrence.

A CommonJS `require` of `fs`, `fs/promises`, `fs-extra`, a glob package, or a
configured enumerator whose uses cannot be followed is reported on its `require`
line as an unresolved root. A computed `require(moduleName)` is also reported:
the module could expose enumeration, so it cannot pass silently. These fallback
diagnostics are replaced by a resolved direct scan through a recognized static
binding. An unknown direct member scan at the load's own offset is reported once.
Unknown export containers, including `fs.promises`, retain the fallback until
their enumeration uses are accounted for. Configured descendant exports also
keep their container visible.

Node loader forms also retain an unresolved-root fallback at the load itself:

```ts
module.require("node:fs");
import { createRequire as make } from "node:module";
make(import.meta.url)("node:fs");
const load = make(import.meta.url);
load(moduleName);
```

This matcher recognizes global `module.require` and `module["require"]`,
and `createRequire` from `module` or `node:module` through named ESM aliases,
namespace/default members, or ordinary CommonJS named/namespace imports.
Direct factory-return calls and same-file initializer aliases of factories and
loaders are recognized through static expressions and cycle-safe initializer chains.
Literal enumerator modules and computed module arguments are reported. Literal
unrelated modules, custom factory imports, and ordinary shadowed bindings are
not treated as Node loaders. A potentially Node-origin binding retains the
fallback after reassignment, namespace mutation, or escape; this is uncertainty,
not a claim that the runtime call still invokes Node's loader.

These Node loader forms do not establish bounded paths, even when a later scan
names `fixtures`: use a statically named import for that proof. Their findings
use the same `unresolvedRoot`, severity, custom message, include/exclude, and
suppression settings. Dynamic property names, factory reexports, arbitrary
wrapper factories, and `.call`/`.bind` indirection are outside this matcher;
rewrite them to a direct named import or a direct Node factory initializer.

Provenance is syntactic. The rule follows `join`, `resolve`, `normalize`,
`dirname`, `fileURLToPath`, `new URL(..., import.meta.url)`, `import.meta.dirname`,
`__dirname`, `process.cwd()`, `tmpdir()`, `mkdtemp`, template literals, and
immutable `const` aliases, including a `const` glob options object with a `cwd`.
Parent traversal is resolved, so `fixtures/../src` is a live path, not a fixture
path. A generic helper such as
`listFiles(dir: string)` is an unresolved root at its enumeration unless every
reachable caller passes an owned root or a module- or `cwd`-anchored fixture
path through an imported binding (see
[Owned parameters and env vars](#owned-parameters-and-env-vars)); a caller that
passes a relative literal, a variable, or nothing keeps the finding. Otherwise
enumerate with a path the helper builds itself from a fixture root, or suppress
the helper.

## Owned parameters and env vars

Two provenance gaps resolve when every route into the enumeration is proven.

**Parameters.** `sync(user, dir = DEFAULT_DIR)` enumerates `dir`. If every
reachable call site passes an owned or fixture root at that argument position,
the enumeration is accepted for those calls. A call that omits the argument
resolves to the default, so one omitting caller keeps the finding. The callee
must enumerate the parameter itself, in the function that declares it, and must
not reassign it. Only one shape is accepted; anything else keeps the finding, because a caller the
call graph cannot see may pass anything:

- The callee is a top-level function declaration or function-valued `const`, not
  a class or object member.
- Every reference to it is a direct call.
- Every file that consumes its module does so through named static imports
  (`import { sync } from "./sync"`) whose uses are all direct calls. A namespace
  import, a default import, `import()`, `require`, a re-export (`export * from`,
  `export { sync } from`), and `vi.mock`/`vi.importActual` of the module make its
  consumers unknowable. A load whose specifier is not a string literal does the
  same for every module.

A consumer is matched to the callee by resolved file, not by specifier text: the
resolver the dependency graph uses (relative paths, tsconfig `paths` aliases,
workspace packages, `package.json` `exports`) maps each such specifier to a file,
so `import * as m from "@lib"` consumes the file `@lib` points at. A specifier
that is neither a bare package nor resolvable to a file keeps the finding for
every callee. A default import, a barrel re-export, and a type-only import of the
callee also keep the finding. Class and object-literal methods are never
accepted, and the rule does not reach them at all.

Callers are the repository call sites in the canonical call graph; only calls
through an imported binding record their argument roots. A function reached only
through `import()` or a namespace is not scanned at all unless a resolved call
reaches it.

**Environment variables.** A getter such as
`() => process.env.X ?? "./tmp/default"` (same file, no parameters, one returned
expression), a `process.env.X` or `process.env["X"]` read, and `path.join`
around it carry the variable through. The enumeration is accepted when:

- every write to `X` in the analyzed files (`process.env.X = ...`,
  `process.env["X"] = ...`, `vi.stubEnv("X", ...)`) assigns an owned root, and
- an unconditional owned write dominates every call chain that reaches the
  enumeration: a module top-level statement before the call, a statement earlier
  in the same function body, or one in a `beforeAll`/`beforeEach` callback that
  covers an `it`/`test` callback calling it.

An owned root is only owned below itself: a path that climbs out of it with
`..` (`join(dir, "..", "src")`) keeps the finding for both parameters and env
vars.

The env read must happen at the call: a value captured into a module- or
function-level `const` (`const dir = process.env.X ?? "."`) keeps the finding, as
does a `beforeAll`/`beforeEach` registered inside a branch.

The finding is kept when the write comes after the call, sits in an `if`, loop,
or `try`, or is overwritten with a live value elsewhere. Writes are read from
every file in the run, including Vitest setup files that the rule's `include`/
`exclude` filters remove from reporting, because a setup `beforeEach` runs after
a test file's `beforeAll`.

Recall comes first: an owned variable stays owned only while every `process.env`
use is understood. A use is understood when it is a member read
(`process.env.X`), a recorded write, a spread copy (`{ ...process.env }`), the
`in` operator (`"A" in process.env`), or an object-destructuring read
(`const { A } = process.env`). Any other use poisons every variable and keeps the
finding: aliases and bare passing, `process["env"]`, `globalThis.process`,
imports or loads of `process`, and destructuring, loop, or update targets.

`process` is accepted only as the bare global identifier, and only through
`process.env` plus a short list of plain members (`cwd`, `platform`, `argv`,
`pid`, `version`, `versions`, `arch`, `hrtime`, `nextTick`, `uptime`, `exitCode`,
`memoryUsage`, `stdout`, `stderr`) or `typeof process`. Any other member
(`getBuiltinModule`, `mainModule`, `binding`), any computed member, and `process`
used as a value poison. Any reference to the global object (`globalThis`,
`global`, `window`, `self`) poisons unless it is `typeof` or a direct read of a
host API on a short list (`fetch`, `console`, timers, `performance`, `URL`,
`Buffer`, `crypto`, `document`, `location`, `navigator`, storage). Every `.constructor` or
`.__proto__` member, a computed key spelling them, and a computed key built with
string concatenation, a template, or a call poison, because they reach the
`Function` constructor. A module load poisons when its specifier is `process`,
`node:process`, `vm`, `node:vm`, or is not a string literal: `require`, `import()`,
`module.require`, `vi.importActual`, `vi.importMock`, `jest.requireActual`, and
`jest.requireMock`. So does an `import` or `export ... from` of those modules.
`createRequire`, `eval`, and `Function` poison wherever they appear, as does any use
of `require` or `module` other than a call, `require.resolve`, or `module.exports`.
The string `"constructor"` or `"__proto__"` anywhere in a file poisons, as does
either name used as a property or binding key (destructuring, `Reflect.get`,
property descriptors, a variable used as a key); only a class method named
`constructor` is exempt. A key built through a variable with no such literal
anywhere (for example `"con" + "structor"` held in a variable) is not followed.

Test-runner helpers follow the same rule. Every reference to `vi` or `jest` (and
to a local name it is imported as) must be the object of a direct, non-computed
call to a known helper that cannot touch the environment (`vi.fn`, `vi.mock`,
timer helpers, and similar, or `vi.spyOn` whose first argument is a plain
identifier or member path) or to `vi.stubEnv`, which is recorded as a
write. Computed access of any kind, a cast, an alias, passing `vi` as a value,
`globalThis.vi`, a namespace or default import of the runner, an `import()` of it,
or a helper outside the known list poisons. `unstubAllEnvs` and `unstubEnvs`
poison wherever they appear, because they restore the default between tests.

## Pass-through mocks

`vi.spyOn(fs, "readdir").mockImplementation((...args) => original(...args))`
forwards the caller's arguments, so the forwarding call is not a root of its
own. Only a `spyOn` with a literal name of an enumerating method (`readdir`,
`readdirSync`, `opendir`, `opendirSync`, `glob`, `globSync`, or a configured
`enumerators` export) qualifies; a bare `vi.fn()`, another method, or a computed
name keeps the advisory. The callers of the mocked function are judged where they enumerate. Only an
exact forward counts: a wrapper with any other parameter, an extra or dropped
argument (`original("./x", ...args)`, `original(...args.slice(1))`), or a
rebound rest parameter keeps the advisory; so does any other use of the rest
binding (an index write, `splice`, `push`, `length = ...`, or a nested closure), as does forwarding outside
`mockImplementation`/`mockImplementationOnce`.

## Runtime assets

Production code sometimes reads a shipped directory, such as migration files read
to report migration status. That is runtime behavior, not a repository scan.
Declare such roots in config; no directory name is built in:

```yaml
rules:
  - rule: test-no-live-repository-scan
    scope: repository
    options:
      runtimeAssets:
        - root: backend/assets/migrations
          reason: shipped migration files read to report status
```

A literal path (no non-literal components) at or under a declared root is
reported as an advisory with target `runtime asset` and the reason in its
message. Matching is by whole path segment, so `assets/migrations-src` and the
parent `assets` are still reported, as is a path with a dynamic component. `root`
is relative to the invocation root and may not contain `..`; `reason` is
required. Delete the entry to make the read a finding again.

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
      runtimeAssets:
        - root: backend/assets/migrations
          reason: shipped migration files read to report status
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
- `runtimeAssets` lists shipped asset roots as `{ root, reason }`; see
  [Runtime assets](#runtime-assets). Default: none.
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
