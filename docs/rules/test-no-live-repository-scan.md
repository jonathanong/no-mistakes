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

Without `roots`, the default `[{ vitest: true }]` selector uses projects from
Vitest config files discovered at the repository root (`vitest.config.*`,
`vitest.workspace.*`, and `vitest.projects.*`). A root workspace or projects
file takes precedence over sibling `vitest.config.*` files. Explicit
`tests.vitest.configs` remains authoritative when configured, and no
`tests.vitest` entry is required for default config discovery; this example
names one file. `severity: advisory` is the recommended adoption
default: findings are listed without failing `check`. Remove it once the
repository is clean.

## Why and when

Use this rule when tests read policy from the repository itself (workflow
policy, lint-config shape, file-layout invariants). Run that check once in a
dedicated static CI command and keep the Vitest test for bounded fixtures that
exercise the evaluator. The rule does not ban filesystem access: reading a
fixture, or a directory the test created, is fine.

## What it catches

Starting from the configured roots (every discovered or configured Vitest
project by default), the rule
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

## Reachability

When a test imports a module, its module initialization excludes the body of
`if (import.meta.main)`: that body runs only when the module is the process
entry point. The same applies to `import.meta.main === true` and `&&`
conditions requiring either form, such as
`if (import.meta.main && shouldReport())`.

The `else` branch, statements after the guarded block, and the body of
`if (!import.meta.main)` remain reachable on import. A condition using `||`
does not establish an entry-only body. Calls in the condition remain analyzed.
An exported scanning function still produces a finding when a test calls it
directly, even if the module also calls it inside an entry-point guard.
The canonical graph retains entry-point calls for policies that explicitly
select an entire file, such as `forbidden-calls` with `traversal: file`.

## Allowed input

An enumeration passes when its root is provably bounded:

- Any path segment equals a `fixtureDirs` entry (`fixtures` or `__fixtures__` by
  default), such as `join(import.meta.dirname, "fixtures", name)`. This includes
  `.github/fixtures`; the directory name is the boundary, regardless of depth.
  Thus `.github/fixtures/x` is allowed, while `.github/fixtures/../workflows`
  is reported: normalization removes the `fixtures` segment before matching.
- The root is anchored at the result of `mkdtemp`/`mkdtempSync`, held directly,
  through a `const`, or through a `let` whose every write is such a root.
  Same-file zero-argument helpers with no reassignment may create local bindings before their final
  return, including an awaited `mkdtemp` in a nested async helper. Owned paths
  returned as object properties can be read through `const` destructuring.
  `mkdtempDisposableSync` results provide an owned root through their `.path`
  property, including a `using` binding. Destructuring needs direct literal fields without
  defaults, rest properties, computed keys, or spreads.
  The classic factories must resolve to Node `fs` or `fs/promises` imports
  (`fs` and `node:fs` specifiers are equivalent). Named imports, renamed
  imports, direct `require` bindings, and stable module receivers are recognized.
  A `promises` binding from Node `fs` is also recognized as `fs/promises`.
  A stable Node `fs.promises.mkdtemp` call is recognized; `fs.promises.mkdtempSync`
  is unresolved because the promises API has no synchronous factory.
  Static CommonJS selections such as `require('fs').mkdtempSync` and
  `require('fs').promises.mkdtemp` keep their Node factory identity when bound
  locally or called directly. Unknown selections, shadowed loaders, and
  mutations or opaque escapes of a Node factory view stay unresolved. A genuine
  `vi.spyOn(require('fs'), 'readdirSync')` and a genuine spy on
  `fs.promises.readdir` leave the factory identity intact.
  Direct `require(...)` calls may still produce the existing conservative loader
  advisory even when an enumerator's temporary root is proven owned.
  A same-named local function, parameter, or import from another module does not establish an
  owned root. A mutated or escaping module receiver is unresolved. Use a
  direct Node import when creating the directory.
  A statically bound Vitest or Jest spy on an unrelated member such as
  `readdirSync` preserves factory ownership; a factory spy does not.
- The root is a parameter, or an environment variable, and the test supplies an
  owned root (see [Owned parameters and env vars](#owned-parameters-and-env-vars)).
- The call forwards a mock's own arguments unchanged
  (see [Pass-through mocks](#pass-through-mocks)).
- The read is under a declared [runtime asset](#runtime-assets); it is listed
  separately instead of failing.

`readdirSync(tmpdir())` and a predictable child such as
`readdirSync(join(tmpdir(), "policy"))` are reported: the shared temporary
directory is not owned by the test. Create a fresh directory with `mkdtemp`
instead. A trusted Node `os.tmpdir()` or `tmpdir()` call can use the same
ownership proof as a direct environment read when every reaching test establishes
its primary environment variable as an owned, nonempty root before the scanner
runs. The primary variable is `TEMP` on Windows and `TMPDIR` on POSIX. Owning only
`TMPDIR` does not prove a Windows temporary root, and owning only `TEMP` does not
prove a POSIX root. Later environment fallbacks remain unproven.

This requires a static `os` or `node:os` import (named, default, or namespace)
or an immutable CommonJS binding; a local function or object merely named
`tmpdir` does not gain environment ownership. A live, missing, late, or mixed
assignment keeps the shared-directory finding. For portable tests, create one
`const fixture = await mkdtemp(join(tmpdir(), "owned-"))`, then call both
`vi.stubEnv("TMPDIR", fixture)` and `vi.stubEnv("TEMP", fixture)` before the scanner.
Direct `process.env.TMPDIR` reads always use `TMPDIR`, regardless of host platform.
`dirname(root)` escapes an opaque owned root and remains reported, including
nested `dirname` calls. `dirname(join(root, "child"))` stays owned because it
removes a known child rather than crossing the root boundary. This applies to
Node `tmpdir()` calls and direct environment reads. An environment assignment
from the parent of a fresh `mkdtemp` root also fails the ownership proof.
A matching `fixtureDirs` segment remains an explicit allowance.

## Unresolved roots

When the root's provenance cannot be established, the rule reports it with the
reason `unresolved root` instead of passing silently. Parameters and env values
are accepted when every selected test caller proves an owned root as described
below. A mixed or unresolved caller, an unowned reassignment, an unmodeled call
or spread, or an unresolved enumeration receiver retains the finding. These go to the non-failing `advisories` list by default. Set
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
including optional calls and optional members (`module.require?.("node:fs")`,
`module?.require("node:fs")`, `createRequire?.(import.meta.url)?.("fs")`),
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
A relative literal glob pattern uses that bounded `cwd` proof. An unknown
pattern remains advisory even with an owned `cwd`: absolute glob patterns can
ignore that directory. In `patterns.flatMap(pattern => globSync(pattern,
{ cwd: directory }))`, the callback parameter has no proven array-element
provenance through the caller. The saved `glob-options-flow` case therefore
remains advisory even when the visible caller supplies a relative pattern array.
Use direct literal patterns so the analysis can prove their root.
Parent traversal is resolved, so `fixtures/../src` is a live path, not a fixture
path. A generic helper such as
`listFiles(dir: string)` is an unresolved root at its enumeration unless every
reachable caller passes an owned root or a module- or `cwd`-anchored fixture
path through a named call (see
[Owned parameters and env vars](#owned-parameters-and-env-vars)); a caller that
passes a relative literal, an unproven variable, or nothing keeps the finding.
When that proof fails, the advisory names the blocker and where it is:
the caller call site and its argument (a live path, an omitted argument, an
unproven variable, or a value forwarded through another parameter), or the
route that made callers unknowable (a namespace import, a default import,
`import()`, `require`, `vi.mock`, a reference other than a direct call, a
callee that is not a named function, or a reassigned parameter), with that
file and line. For a static literal-object property, the diagnostic names the
property's value and its own line, followed by `as property` and the selected key.
For example, a multi-line `{ include: [...], root }` caller names `root` on the
shorthand property's line rather than truncating the outer object. A positional
argument likewise uses the argument expression's line. Opaque object shapes
retain the outer argument as context. When several modules in the test-root closure are unreadable,
or several `setupFiles` entries do not resolve, the advisory names the first
and the count of the rest. A Vitest config that cannot be read or does not
match the selected tests is a different cause from an unresolved `setupFiles`
entry. The fix tells the caller to pass an owned root. It does not
ask a production helper to build the root itself. Otherwise
enumerate with a path the helper builds itself from a fixture root, or suppress
the helper.

## Owned parameters and env vars

A statically named imported helper may return the environment root, such as
`export function getLocalDir() { return process.env.X ?? "./tmp/data"; }`.
The helper must have no parameters and a single direct return of `process.env.X`,
optionally using `??` or `||` with a string literal fallback. Path operations
inside the imported helper remain unproven. The canonical call graph resolves
its source; arbitrary imports, captured helper results,
side effects in the helper, and parent escapes retain unresolved-root findings.
This includes `readdir(path.join(getLocalDir(), table))` when a hook establishes
the environment root and every reaching caller supplies a provably relative
child suffix, including a known constant or a safe string literal default.
Unknown, escaping, or absolute parameter values remain unresolved. Static child
and parent components normalize before this proof.

Hook names must resolve to Vitest named imports (including aliases) or
Vitest globals with no local binding. Local functions or custom imports with those names
do not establish lifecycle ownership.
Unconditional same-scope `beforeEach` setup may pair with `afterEach` cleanup,
and `beforeAll` setup may pair with `afterAll` cleanup. Supported cleanup is a
named `delete process.env.X` or a direct `vi.unstubAllEnvs()` through the named
Vitest import. The owned write must still run before every reaching test call.
A `beforeAll` write followed by an `afterEach` reset cannot prove later tests;
conditional or live writes, arbitrary reset references, and scans from cleanup
callbacks remain unresolved. Concurrent callbacks cannot rely on shared
environment ownership when cleanup in their file, imports, or matched setup
project may overlap. Use a local `mkdtemp` root or sequential tests. Unrelated
sequential callbacks keep their ownership proof. This includes concurrency selected
by a matched project's `test.sequence.concurrent`; unknown configured values
remain conservative, and an unrelated sequential project keeps its own proof.
Runner-level reset configuration stays conservative.

A same-file helper such as `const getDataDir = () => process.env.DATA_DIR ??
"./tmp/data"` keeps the environment origin in `readdir(join(getDataDir(), table))`.
The fallback remains the production root: tests must assign `DATA_DIR` to an
owned root before every reaching call, for example
`process.env.DATA_DIR = await mkdtemp(join(tmpdir(), "case-"))`. A late assignment,
a caller with no assignment, or an assignment from `process.cwd()` cannot prove
ownership. `vi.stubEnv("TMPDIR", fixture)` supplies the same proof when `fixture`
is an owned `mkdtemp` result. A nested `mkdtemp(join(parent, "impact."))` creates
a fresh owned directory even when `parent` comes from that environment.

When an environment root is unresolved, the finding should name the environment
variable and retain the path expression that reaches the enumerator, including
its fallback. If a test clears that variable before importing a module-level
scanner or calling a helper, identify the selected test file and clearing line:

```typescript
// scanner.mts
const dir = process.env.EXAMPLE_LOCAL_DIR ?? "./tmp/example";
export const entries = readdirSync(dir);

// tests/scan.test.mts
it("uses the fallback", async () => {
  delete process.env.EXAMPLE_LOCAL_DIR;
  await import("../scanner.mts");
});
```

The finding should point to the enumeration and explain that clearing
`EXAMPLE_LOCAL_DIR` selects `"./tmp/example"`; it should also point to the
`delete` in `tests/scan.test.mts`. When no owned write is visible from a selected
test call, name that variable and call site and say that ownership is not proven
there. Do not claim the variable is never assigned when an unresolved or
unselected setup could still write it. A computed fallback such as
`join(process.cwd(), "tmp", "example")` remains unowned too, and should be
shown as the fallback expression. An unconditional owned `beforeEach` write
followed by `afterEach` cleanup remains a valid control: the cleanup happens
after the scan and does not make the earlier root unsafe.

When selected tests have mixed environment histories, report the history that
reaches the unsafe scan. If one test scans with the owned `beforeEach` value and
a later test deletes the variable before scanning, identify the later test's
delete and call lines instead of describing the whole suite as missing an
assignment. Likewise, a module-level delete superseded by an owned setup before
every scan is not the blocker; explain the remaining unresolved path expression
or fallback rather than attributing it to that earlier delete.

Two provenance gaps resolve when every route into the enumeration is proven.

**Parameters.** `sync(user, dir = DEFAULT_DIR)` enumerates `dir`. If every
reachable test call site passes an owned or configured fixture root at that argument position,
the enumeration is accepted for those calls. Same-file call chains and recursive
helpers retain this proof only when every external entry supplies a bounded root;
a recursive call cannot establish ownership by itself. A call that omits the
argument resolves to the default, so one omitting caller keeps the finding. The callee
must not reassign the parameter. Derived paths, nested helpers capturing the
parameter, and an options parameter read through a static property such as
`options.root` can retain the caller proof. `join(repoRoot, ".x/a")` and
`resolve(source)` keep that parameter. `fn(join(root, "rules"))` is an owned root
when `root` is an owned temporary directory. A self-recursive enumerator stays
proven when every external entry is owned.

A same-file helper may return that owned root, including a helper whose parameter
is only the `mkdtemp` prefix. The returned origin must stay a non-dynamic temporary
directory and must not climb out of it with `..`. A dynamic return that drops a
parameter, including `join(mkdtempSync(...), suffix)`, stays unresolved. The test
may destructure the returned object (`const { source } = mkFixture()`). An
object-pattern parameter (`function ensure({ source } = {})`) uses the same
literal-object caller proof as `options.source`. A destructured default is ignored
while the property's own origin is proven. When a literal object omits the key and
has no spread, the default expression is the origin. An unknown object, a spread,
or a present property with an unknown value stays unresolved, so a live default
that might still apply keeps the finding. A `const` object is followed only when
every other use is a non-mutating member read. A property assignment or any other
escape stays unresolved. A reachable caller that omits a defaulted
argument also keeps the finding. A direct object property is one key deep: nested
patterns, computed keys, and rest bindings stay unresolved. A member read of an
object-pattern parameter, such as `function scan({ opts }) { readdirSync(opts.root) }`,
stays unresolved too. The pattern key is not replaced by the accessed property, so
an owned sibling `root` cannot clear a live `opts.root`. A property beside a
rest binding can still be proven. Production modules that
no selected test imports do not add callers. An unreadable module that no
test, `setupFiles`, or `globalSetup` reaches does not erase a caller proof,
including a lint config such as `.dependency-cruiser.cjs` loaded only from the
runner config. The same unreadable module imported by a test that reaches the
function keeps the finding.

An object argument must expose the requested property in a literal object, without
mutation or escape. Method and
constructor calls on that object prevent this proof because they may mutate its
properties. A member written through an object or array assignment target,
including nested patterns, rest elements, defaults, and loop targets, also
invalidates ownership. For example, `({ root: options.root } = { root: process.cwd() })`
before `readdirSync(options.root)` remains reported even when the caller supplied
an owned root. Reads in default initializers or computed property keys do not
write the member. Anything else keeps the finding, because a caller
the call graph cannot see may pass anything:

- The callee is a named function declaration or function-valued `const`, not
  a class or object member.
- Every reference to it, and to a function that reaches it, is a direct call.
  A non-call reference to a sibling const export, such as a string used in a
  template literal, does not count. A property name on a member access
  (`import.meta.main`, `obj.main`) is not a reference to a binding named
  `main`. A re-export specifier
  (`export { enumerate, other }`) or plain `export *` does not count either: importers of that
  barrel, including through further re-export chains, are judged at their
  call sites. Type-only references such as `void (0 as unknown as typeof scan)`
  and discarded values such as `void (scan as unknown)` do not escape the
  function. Runtime arguments such as `consume(scan as unknown)` still do.
- Every test-reachable file that consumes its module does so through named static imports
  (`import { sync } from "./sync"`) whose uses are all direct calls. A namespace
  import, a default import, `require`, a namespace re-export (`export * as ns`),
  and `vi.mock`/`vi.importActual` of the module make its consumers unknowable.
  A literal dynamic import whose result is discarded, or whose awaited
  namespace binding is unused, does not expose every export. An unexported
  promise binding whose only reads discard its value with `void` is also inert. Proven named
  reads select only those exports through the canonical graph. A namespace
  passed to unknown code, a computed member read, an exported namespace, or an
  escaping import promise retains the conservative advisory. Method calls on
  a namespace narrow only when the selected exports are proven independent
  of their receiver.
  A named value import that is not a call blocks only the export it names.
  A rename (`import { fn as alias }`) still names that export.
  A named or plain-star barrel re-export preserves the route when all consumers
  remain named direct calls. A load through a `const` string specifier is
  attributed to that module. A specifier that cannot be resolved statically,
  or any other module whose loads this pass cannot read, blocks caller proof
  only for enumerating functions that module can reach from a test root. That
  set is the `setupFiles`, `globalSetup`, and test files found by reverse
  load edges from the function's module, plus the import closure of those
  files. An unreadable module outside that closure does not block the proof.
  An unreadable module inside it still does. The advisory names the deepest
  import that could not be read, at that import's own file and line, not the
  first edge of the importing module. A specifier that resolves to a readable
  file is not that failure. That includes a workspace package imported by its
  name when `package.json` `exports` maps `"."` and `"./*`, and a relative
  `.mts` import of a file that exists. The check uses the same session
  resolver as the dependency graph, and follows a relative or package entry
  that exists on disk. A file that exists is still unreadable when a load
  inside it cannot be named, such as `require` of a non-literal specifier.
  That failure stays on the load's line. When the load edges show a path, the advisory includes
  it from a test or setup root. A path string, `require.resolve`, or a CLI
  invocation does not put a module on that path. When no unreadable module is
  in the closure and caller proof still fails at the function, the advisory
  names a recursive inner helper only when the judged root itself flows
  through the recursive argument. An outer parameter captured by an inner
  helper, while that helper recurses on a relative suffix, is traced to the
  outer function's callers, following forwarded parameters until the first
  unproven argument. For example, when `listFiles(root)` captures `root` in
  `visit(rel)`, `update(root)` calls `listFiles(root)`, and a test calls
  `update(target)`, the advisory names the test's `update(target)` call and
  `target`. A file-wide consumer without a named route still retains the
  advisory when all visible callers pass owned roots. The diagnostic includes
  the unproven call's file, line, and argument text. It does
  not stop at the function declaration when a caller exists, and it does not
  describe that declaration as an unreadable module.

A consumer is matched to the callee by resolved file, not by specifier text: the
resolver the dependency graph uses (relative paths, tsconfig `paths` aliases,
workspace packages, `package.json` `exports` including a single `*` subpath
such as `"./*": "./*.mts"`) maps each such specifier to a file,
so `import * as m from "@lib"` consumes the file `@lib` points at. A specifier
that is neither a bare package nor resolvable to a file keeps the finding for
every callee whose test-root closure includes the module that named it. A default
import or a namespace re-export also keeps the finding; erased type-only imports
and re-exports do not. Class and object-literal methods can be
reached from a file root; when a method's enumeration root depends on an
unresolved parameter or expression, the rule reports an unresolved-root
advisory at that call.

Callers include the selected tests and their selected `setupFiles` execution,
using the canonical call graph. A setup-only scan is checked, and a setup call
with a live root prevents an otherwise owned test caller from clearing it.
Production-only callers do not widen that set. This includes a guarded dynamic
import from an entrypoint that no selected test or setup reaches. The same
import remains an unproven caller when a test, setup, global setup, or selected
runner configuration can reach it. Unknown runner configuration keeps the
conservative whole-inventory fallback for environment writes without admitting
unrelated production importers as caller evidence. Only calls
through imported bindings or same-file named functions record their argument roots.
A function reached only through `import()` or a namespace is not scanned at all unless a resolved call
reaches it.

**Environment variables.** A getter such as
`() => process.env.X ?? "./tmp/default"` (same file, no parameters, one returned
expression), a `process.env.X` or `process.env["X"]` read, and `path.join`
around it carry the variable through. The enumeration is accepted when:

- every write to `X` in the relevant test execution files (`process.env.X = ...`,
  `process.env["X"] = ...`, `vi.stubEnv("X", ...)`) assigns an owned root, and
- an unconditional owned write dominates every call chain that reaches the
  enumeration: a module top-level statement before the call, a statement earlier
  in the same function body, or one in a same-file `beforeAll`/`beforeEach`
  callback that covers an `it`/`test` callback calling it. An unconditional
  module-level write in an eagerly imported setup module also dominates the
  importing module body, including through a chain of static imports.

A same-test `vi.stubEnv` can prove a smaller synchronous interval even when
teardown restores env or another test writes the same key:

```typescript
it("scans an owned environment", () => {
  const root = mkdtempSync(join(tmpdir(), "x-"));
  vi.stubEnv("OWNED_SCAN_ROOT", root);
  scanOwnedEnvironment();
});

export function scanOwnedEnvironment() {
  return readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
}
```

This proof requires a named `vi` import from `vitest`, an unconditional stub,
and the immediately following statement calling a named helper without
arguments. The helper must have no parameters and a single return statement
whose root reads `process.env` directly. Every caller must satisfy that proof.
A selected dynamic import that passes this helper to other code is an unproven
caller; a selected reference to a sibling export does not block the helper.
No other call or `await` can run between the stub and the read, so an
`afterEach` reset, `unstubEnvs`, or another test's env value cannot interrupt it.
Conditional stubs and `vi.stubEnv("OWNED_SCAN_ROOT", process.cwd())` retain
advisories. Resetting in an argument, helper, or getter, passing an unknown
`vi` alias, and methods merely named `stubEnv` do not establish this proof.

Otherwise, the whole-run proof above applies. Both direct assignment and `vi.stubEnv` carry owned provenance across these
static setup imports:

```typescript
// setup.mts
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { vi } from "vitest";
vi.stubEnv("OWNED_SCAN_ROOT", mkdtempSync(join(tmpdir(), "scan-")));
// Equivalently: process.env.OWNED_SCAN_ROOT = mkdtempSync(join(tmpdir(), "scan-"));
```

A test can import that setup before calling a named source helper that reads
`process.env.OWNED_SCAN_ROOT ?? ""`. A statically configured Vitest `setupFiles`
module also supplies this proof when every matching project supplies an owned
write. A same-file const whose `setupFiles` array is a literal, including one
wrapped in `satisfies`, `as`, or parentheses and spread as
`setupFiles: [...baseProject.test.setupFiles]`, resolves to those paths. The
existing prepared project catalog supplies setup ownership; configs
are not parsed again. Only unconditional module-level writes in setup files or their
static imports qualify. `globalSetup`, unknown setup paths, imported hooks,
mocked setup modules, unexecuted functions, conditional or dynamic imports, and import cycles that
can reach the caller before the write retain the advisory. A write after an
earlier call in the setup file does not cover that call. An env value without a stub
or any write naming a repository path also retains the advisory.

An owned root is only owned below itself: a path that climbs out of it with
`..` (`join(dir, "..", "src")`) keeps the finding for both parameters and env
vars. A temporary-root write is owned only while its value stays inside the
temp directory. A fixture write is also owned when its
normalized segments retain a configured `fixtureDirs` entry; for example,
`join(import.meta.dirname, "../samples")` is bounded when `samples` is configured. `process.env.X = join(mkdtempSync(...), "..", "..", "..")` is not an
owned write, and that write does not dominate the call. The configured segment
allowance also applies when the prefix is unresolved; a segment outside that list
does not establish ownership.

A named helper can receive `process.env.X` as an argument when the value is
proven owned at that argument read. The same dominance and poison checks apply
to direct calls and named forwarding helpers.

The env read must happen at the call: a value captured into a module- or
function-level `const` (`const dir = process.env.X ?? "."`) keeps the finding, as
does a `beforeAll`/`beforeEach` registered inside a branch.

A hook setup-and-restore pair remains unresolved: assigning an owned root in
`beforeAll` and restoring the prior value (or deleting the variable) in
`afterAll` does not establish that every write is owned. The analysis does not
prove a lifecycle interval for every imported caller, concurrent test, and hook;
it cannot discard the restore write merely because a visible caller is in an
`it` callback. Pass the owned root as an argument to a statically named helper
instead. The unresolved-root hint names two supported rewrites: build a `const` fixture
path or owned `mkdtemp` root inside the enumerating function, or supply it through
every statically resolved test caller (`vi.stubEnv` for env roots). A reasoned
suppression is the last option.

The finding is kept when the write comes after the call, sits in an `if`, loop,
or `try`, or another relevant write is live. The proof includes selected test roots,
imported module initialization, reached helpers, runner configuration and setup
files, even when `include`/`exclude` removes them from reporting. A setup
`beforeEach` can run after a test file's `beforeAll`.

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
anywhere (for example a name assembled from two fragments held in a variable) is not followed.

Test-runner helpers follow the same rule. Every reference to `vi` or `jest` (and
to a local name it is imported as) must be the object of a direct, non-computed
call to a known helper that cannot touch the environment (`vi.fn`, `vi.mock`,
timer helpers, and similar, or `vi.spyOn` whose first argument is a plain
identifier or member path) or to `vi.stubEnv`, which is recorded as a
write. Computed access of any kind, a cast, an alias, passing `vi` as a value,
`globalThis.vi`, a namespace or default import of the runner, an `import()` of it,
or a helper outside the known list poisons. `unstubAllEnvs` and `unstubEnvs`
poison wherever they appear, because they restore the default between tests.
`stubGlobal("process", ...)` poisons too, including a named import, a member, a
string computed member, and an optional call, even when the receiver is not
named `vi`. `stubGlobal` with a first argument that is not a string literal
poisons. `stubGlobal("fetch", ...)` and ``stubGlobal(`location`, ...)`` do not,
by themselves.

A computed object key poisons when its text is the literal `unstubEnvs` or
`unstubAllEnvs` (a string or a template with no substitutions). Any other
computed key whose text is not a literal poisons in a selected project config
or its imported modules, including custom-named configs. Unknown config or
setup selection retains the conservative fallback to the graph file inventory.
A literal key with different text, such as `"include"`, does not. The same two
names poison wherever they appear in a Vitest JSON or JSONC config: each path in
`tests.vitest.configs`, plus `vitest.workspace.json`, `vitest.workspace.jsonc`,
`vitest.projects.json`, and `vitest.projects.jsonc` at the project root. A JSON
or JSONC workspace file inside a package is not read unless that path is listed
in `tests.vitest.configs`. A missing file does not. A file that exists but
cannot be read or parsed does.
The key poisons whatever its value is, including `false`.

`process.env` forms that poison every variable:

- an alias (`const env = process.env`) or passing `process.env` where it is not a member read, a spread copy, an `in` test, or an object-destructuring read
- `process["env"]`, `globalThis.process`, and any `process` use other than `process.env`, the plain members listed above, or `typeof process`
- an import, re-export, `require`, or `import()` of `process`, `node:process`, `vm`, or `node:vm`, or a load specifier that is not a string literal
- a destructuring, loop, or update target on an env member
- assigning `process.env` itself, or a computed env write whose key is not a string literal
- `delete process.env` without a name, and `Object`/`Reflect` mutators applied to `process.env`
- `vi.stubEnv` / `jest` stub helpers whose name is not a string literal
- `unstubAllEnvs`, `unstubEnvs`, and `createRequire` wherever those names appear
- `stubGlobal("process")` and `stubGlobal` with a non-literal first argument
- a computed `unstubEnvs` / `unstubAllEnvs` key, and an opaque computed key in a selected config or its imports
- `unstubEnvs` or `unstubAllEnvs` in a Vitest JSON or JSONC config, as above
- `.constructor` / `.__proto__`, a computed key that spells them or is built from strings, templates, or calls, and the strings `"constructor"` and `"__proto__"`
- `eval` and `Function`
- `require` or `module` used other than as a call, `require.resolve`, or `module.exports` / `module.require`
- `vi` or `jest` (including a local import name) used other than as a known helper or `stubEnv`
- a namespace or default import of `vitest` or `@jest/globals`, or an `import()` / `require` of those modules

## Small and large projects

A standalone helper and the same helper in a larger project use the same caller
proof. Unrelated source files cannot invalidate it merely by containing an
opaque module load or a same-named function used as a value. Earlier analysis
collected those blockers across the whole file inventory, so adding either kind
of unrelated file made an otherwise owned parameter appear unresolved.

The proof now uses the selected test execution scope and keys local function
escapes by file. A selected test file without calls still contributes its writes
and module loads. Selected setup calls count as callers, and config evidence
comes from the prepared project selection and its imports rather than filename
conventions. Imports, reexports, runner settings and setup modules retain
their conservative checks; an unknown consumer in that scope keeps the finding.
Selected setup and config sources must parse successfully. Otherwise, `check`
reports incomplete analysis; fix the reported source parse error before relying
on caller ownership.

Long call chains use a cycle-safe traversal of the canonical graph without a
fixed caller-state cutoff. Cycles need an independently owned entry argument.

## Pass-through mocks

A wrapper may capture a Node filesystem enumerator in a `const`, replace that
same member with a rest-argument function, and return the captured function's
unchanged arguments. Read-only strict-equality checks on an argument are allowed
for fixture failure injection. The ordinary filesystem call still receives root
analysis. Mutating or escaping the argument array, replacing the arguments,
unknown receiver writes, and competing aliases of the same filesystem module
retain unresolved receiver findings. Extra opaque calls or constructors also
keep the receiver unresolved; failure injection may throw global
`new Error` with primitive literal arguments when `Error` has no local binding.

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

Ordinary root provenance can independently prove a rewritten argument. For example,
an unconditional `args[0] = ownedRoot` before forwarding passes when `ownedRoot`
is a fixture path or owned temporary root. This does not use the identity-only
mock exemption.

A conditional rewrite such as `if (args[0] === migrations) args[0] = root`
before `original(...args)` remains unresolved. This is no longer identity
forwarding: the analysis does not connect every mocked caller to the rewritten
argument tuple, and other argument values pass through unchanged. A
`runtimeAssets` declaration for `migrations` accounts for the visible asset read
but does not prove that the forwarding call receives only bounded roots. Use
an explicit bounded path in the replacement implementation, or suppress that
forwarding call with the reason its callers are safe.

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
  which uses discovered Vitest configs; `tests.vitest` configuration is optional.
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

Argument forwarding preserves each known position and property. Literal array
spreads and a helper's rest parameter can carry an owned root to an enumerator.
For object spreads, the last property wins: `{ ...inputs, outputRoot: root }`
retains the explicit owned root, while `{ outputRoot: root, ...inputs }` stays
unresolved when `inputs` is unknown. Object rest destructuring proves excluded
keys: in `main({ outputRoot, ...inputs })`, spreading `inputs` after the explicit
`outputRoot` cannot replace it. A real override or an opaque spread retains the
finding. Writes to a rest slot are followed in source
order; a conditional write also retains the unchanged input alternative. A slot
write after an enumeration in the same loop keeps the root unresolved because
the next iteration can read that write. Pass an owned root directly at the
enumeration to avoid relying on mutable forwarded arguments across iterations.

A destructuring default applies only when a caller demonstrably omits that
property. A whole-parameter default other than `{}` is not used to prove a
destructured property default, and forwarding an object parameter with its own
default keeps the property unresolved. Pass an explicit owned property to make
that input provable. Passing an explicit owned `schemaDir` proves
`function main({ schemaDir = process.cwd() } = {})`; omitting it retains the live
default. An unknown object or an explicitly unknown property is unresolved.
A fallback on a parameter property, such as `paths.schemaDir ?? DEFAULT`, is
checked against each reachable test caller. An explicit owned property supplies
the root for that call; a missing, nullish, live, or unknown property cannot
inherit that proof. Production call paths that the selected tests never execute
do not supply additional arguments to that proof.

Missing bounded argument facts do not establish omission: defaults require a
known argument count or a known missing property on a closed object.
Fallbacks that cannot be selected from proven caller properties retain every
finite alternative, including optional-chain and `||` fallbacks. Each reachable
alternative must be owned, a fixture path, or a known relative suffix resolved
under an owned base. For `resolve(worktreeRoot, project.root ??
project.test?.root ?? '.')`, the base and suffix are proved independently from
the same selected test callers. Closed literal objects establish missing nested
properties; unknown objects do not. Nested literal initializers are followed only
while their containers stay in property-read chains. Nested writes, mutating
method calls, and container escapes through aliases or call arguments keep the
root unresolved. The same restriction applies when an intermediate helper
forwards an object parameter: writes to the requested member or its parents,
and escapes of its containers, invalidate the caller's field proof. Unrelated
property writes and reads of the path string preserve it. Passing the path value
to other code preserves this proof only for a known immutable string. URLs and
buffer-valued filesystem results may be mutated through that reference; convert
a URL with `fileURLToPath` or keep the mutable value unexposed before scanning.
The default string-returning temporary-directory overload retains the string
proof. Literal property keys containing dots are not interpreted as nested
selectors. Absolute, unknown, and climbing suffixes keep the finding. Rewrite
them to literal relative paths or pass an owned root explicitly.
A relative suffix that climbs above the owned directory, or `dirname` of the
owned directory itself, does not remain owned. Accessor-bearing objects and
object literals that set `__proto__` remain opaque to property/default proofs.

## Counterexample

```ts
// tests/policy.test.ts
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const workflows = join(import.meta.dirname, "..", ".github", "workflows");
const entries = readdirSync(workflows).map((name) => readFileSync(join(workflows, name), "utf8"));
```

Reported at the `readdirSync` call: it enumerates a path under the test
module's directory and a Vitest root reaches it.

## Fix

Move the repository-wide check into a released command that runs once in static
CI, and keep the Vitest test for small supplied fixtures that exercise that
check's evaluator. For an unresolved root that is not a parameter, build a
`const` fixture path or owned `mkdtemp` root inside the enumerating function,
or supply it through every statically resolved test caller (`vi.stubEnv` for
env roots). For a parameter the proof cannot close, fix the caller or the
route the advisory names, and pass an owned root from that caller. Configured
`fixtureDirs` apply to caller arguments and env writes too. A mixed or unknown
caller, escaping function, unsafe env write, or unowned reassignment must be
rewritten or isolated before that proof succeeds.

An absolute literal below the invocation root matches `runtimeAssets` using the
same root-relative containment test as a relative literal; paths outside that
root do not inherit the declaration. An unresolved relative consumer in the
selected test scope invalidates owned-parameter proofs and retains the finding.

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
