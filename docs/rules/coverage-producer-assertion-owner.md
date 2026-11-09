# `coverage-producer-assertion-owner`

## Why and when

An opt-in TS/JS project rule for configured coverage-producing callables. It
rejects the known assertion-free class: executing production code alone can
increase coverage without checking an observable result. It does **not** prove
that every covered line has adequate behavioral assertions.

## What it requires

```yaml
rules:
  - rule: coverage-producer-assertion-owner
    scope: repository
    options:
      producers: [{ file: tests/coverage.mts, symbol: produceCoverage }]
      assertions: [{ module: 'node:assert', export: strictEqual }]
      maxDepth: 64
      maxNodes: 10000
      unanalyzable: report
```

`producers` and `assertions` must be non-empty. Producers select exactly one
prepared callable by file and symbol; missing or ambiguous selectors are errors.
There are no framework, test-location, or assertion-name defaults. Assertion
imports match resolved module/export identities, including named imports,
namespace member calls, and imported aliases. For the Node assert specifiers
`assert`, `assert/strict`, `node:assert`, and `node:assert/strict`, a
default-import member call `assert.<export>()` reaches that export, and a bare
`assert()` reaches `ok`. Those specifier strings are not aliases of each other:
configuring `node:assert` does not match an import of `node:assert/strict` or
`assert`. A locally shadowed identifier or a string containing an assertion
name does not qualify. Helpers need no local assertion when their actual call descendants own
one. Actual resolved callback invocations use the canonical call graph. Storing,
registering, or forwarding a callback without invoking it supplies no assertion
evidence; reference-only callback/member edges are excluded from this proof.
Callback inference requires a resolved actual invocation. Different incoming
callback identities which the canonical graph cannot reconcile remain
unanalyzable, even when those known callbacks individually own assertions.

## Counterexample

```ts
// tests/coverage.mts
import { strictEqual } from 'node:assert';
function execute() { return 1; }
export function produceCoverage() { execute(); }
function neverCalled() { strictEqual(1, 1); }
```

## Valid example

Fix by calling an assertion-bearing owner from the producer, directly or through
resolved helpers:

```ts
// tests/coverage.mts
import { strictEqual } from 'node:assert';
import assert from 'node:assert/strict';
import * as strict from 'node:assert/strict';
import { query } from 'pg';

function execute() { return 1; }
export function produceCoverage() {
  query('select 1');
  strictEqual(execute(), 1);
}
export function defaultEqual() { assert.equal(1, 1); }
export function namespaceOk() { strict.ok(1); }
export function bareOk() { assert(1); }
```

`produceCoverage` is the configured producer. Its named `strictEqual` is the
owner, so the unmodeled `query` call does not make it unanalyzable. The same
owner is reached through a default import as `assert.<export>()` and through a
namespace import as `strict.<export>()`. A bare `assert()` selects `ok`.
Configure each module string you import; `node:assert` does not cover
`node:assert/strict`. A producer that only calls `query` stays unanalyzable.

Uncalled sibling/nested functions, assertions in other files without a call
path, literal dead conditional/logical arms and loop bodies, and calls after a sequential `return`/`throw`/`break`/`continue` do not
satisfy the producer. A helper which merely calls the producer is not reachable
in the producer's forward call direction. Branch feasibility beyond these
syntactic exclusions and semantic assertion adequacy remain outside this rule.
When the switch discriminant and every case value are supported literals, only
the matching case (or default) and its fallthrough can supply evidence; direct
or plain-block abrupt exits stop that fallthrough. Unknown switch identities retain the
canonical graph's syntactic treatment.

## Ownership contracts

For an intentionally modeled callback or cross-process dispatch, configure a
genuine ownership contract:

```yaml
ownership:
  - producer: { file: tests/coverage.mts, symbol: produceCoverage }
    owner: { file: tests/contract.mts, symbol: assertWorkerResult }
    via: { module: 'node:child_process', export: spawn }
```

The exact originating callable must actually invoke the configured binding.
The contract adds a canonical `assertion-owner` relationship; the owner must
still reach a configured assertion. A contract or dispatch in a sibling
callable supplies no evidence. Contracts belong to their rule application and
cannot satisfy another application's producers. These are trusted declarations
of real runtime contracts; the analyzer cannot validate a process protocol.

Class construction follows the canonical constructor bridge; unused methods do
not execute merely because their class is constructed. Calling a generator only
creates an iterator and never proves that assertions in its body executed. This
rule does not model iterator consumption, including `.next()` and asynchronous
generators: reaching a generator uses `unanalyzable` policy, even if its import is
configured as an assertion or dispatch selector. Configured generator producers
and generator ownership targets follow the same policy.

Read or parse failures in required producer, referenced helper, or modeled owner
sources always make aggregate `check` incomplete; strict Rust checks return a
source error. Fix the failing source. `unanalyzable: ignore` and suppression
directives cannot hide these failures, even when another assertion is reached.
Independent healthy applications still run.

`maxDepth` (default 64) and `maxNodes` (default 10000, including the producer)
are positive traversal bounds. A reached bound with unexplored work is
unanalyzable. Dynamic targets, unknown alternatives, and Node `fs` callback
shapes also follow shared `unanalyzable` policy: `report`
emits a finding, `advise` emits an advisory, `ignore` explicitly omits that
uncertain result. Those results never supply assertion evidence, but they are
deferred: the rule asks whether some owner is reached, so once any owner is
reached the producer passes even when another call is dynamic, hits a bound, or
is an unmodeled external module call such as a PostgreSQL client or another
Node builtin. With no owner reached, any such uncertainty is reported with
a cause-specific `unanalyzable` message. Standard built-ins are not uncertainty
(see [Built-in calls](#built-in-calls)); external process calls need contracts.

## Uncertainty diagnostics

Human output and JSON `rules[].message` / `advisories[].message` identify each
applicable cause in deterministic order:

- Unresolved dispatch includes the call file, line, exact callee expression,
  and the reason the call is opaque. For example:
  ``unresolved dispatch at src/producer.mts:2 callee `dispatch` (callee target is unknown)``.
  Rewrite dynamic dispatch as a statically resolved call or configure a genuine
  ownership contract. External calls and unmodeled callback execution name the
  corresponding call site too; unmodeled generator bodies name their module and symbol.
- Traversal limits name the option and configured or default value, plus the
  last module and callable reached on the blocked path. For example:
  `traversal limit maxDepth=1; last module src/helper.mts#helper`.
  Inspect that path and raise the named bound when more traversal is intended.
  Both `maxDepth` and `maxNodes` are reported if both block the same step.
- Source failures remain incomplete checks, with the failing file in
  `incomplete[].file` and `source failure in <file>: <error>` in
  `incomplete[].message`. Human output carries the same file and error. Fix the
  source read or parse failure; changing traversal bounds cannot resolve it.

A message names the first five causes in sorted order (file, line, then
position; traversal limits and generators after calls) and ends with `+N more`
when others exist. JSON `rules[]` / `advisories[]` findings also carry
`causes`: up to 25 entries of `{ kind, file, line?, callee?, detail }`, where
`kind` is `unresolved-dispatch`, `traversal-limit`, or `unmodeled-generator`.
Ordinary no-owner findings omit `causes`. A custom `message` is followed by the
same text. Only sites which block the proof are listed.

A producer with multiple uncertain paths reports their causes together. A reached
assertion still passes despite dispatch or traversal uncertainty. Source failures
remain errors even when an assertion is reached. Disabling this rule retains the
same output and does not collect its uncertainty details.

## Built-in calls

Standard ECMAScript built-ins and a small set of Node modules cannot reach a
repository assertion, so calling them is neither evidence nor uncertainty.
Without this model a producer whose closure is ordinary utility code would be
reported as `unanalyzable` for every `JSON.stringify`, `items.map`, or
`join` call even though none can hide an assertion. A reached assertion already
passed regardless; the model changes only which result a producer with no
reached owner receives: the definitive no-owner finding instead of a long list of
unresolved dispatch.

- Global namespaces: `JSON`, `Math`, `Number`, `String`, `Boolean`, `BigInt`,
  `Symbol`, `Date`, `Object`, `Array`, `Promise`, `console`, `Intl`, `RegExp`,
  `Buffer`, `URL`, `performance`; plus `parseInt`, `parseFloat`, `isNaN`,
  `isFinite`, `encodeURIComponent`, `decodeURIComponent`, `structuredClone`.
- Prototype methods (`trim`, `slice`, `split`, `join`, `push`, `has`, `get`, `set`,
  `test`, `getTime`, `log`, ...) only on a receiver which is provably built-in
  in the same file: a string, number, regular expression, template, or array
  literal; an object literal without a member of that name; `new` of a built-in
  constructor (`Map`, `Set`, `Date`, `RegExp`, `Promise`, `AsyncLocalStorage`,
  ...); a built-in factory such as `Array.from` or `Object.keys`; a chain of
  result-preserving methods on such a receiver (`text.trim().split(',')`); or a
  name whose every declaration is built-in, through an annotation (`string`,
  `T[]`, `Map<...>`, ...) or a `const` initializer of the kinds above. A receiver
  whose type is unknown stays `callee target is unknown`, even when the method
  name matches a built-in: a repository `Sink.push` is never mistaken for
  `Array.prototype.push`. A name collision can only keep a producer uncertain,
  never make it a definitive no-owner error. Annotate the binding (`path: string`)
  to make the proof explicit.
- `AsyncLocalStorage` `getStore`, `enterWith`, `disable`, `run`, and `exit`,
  in files which import it from `async_hooks` or `node:async_hooks`.
- Synchronous Node modules `path`, `url`, `os`, `querystring`, `buffer`, and
  `string_decoder` (with or without the `node:` prefix). Other external modules,
  such as an error reporter, remain `external call has no ownership model`.
  This includes namespace bindings followed through local default or named
  re-exports and `export * as` declarations. The receiver must resolve to the
  imported namespace; matching a package or method name alone supplies no
  provenance. Shadowed receivers remain uncertain.

Named imports through a sole plain `export * from 'ext-pkg'` source also retain
the external package/export identity, including through local star barrels.
For example, `import { work } from './barrel.mjs'; work()` reports
`external call has no ownership model` when that barrel only stars `ext-pkg`.
The fallback requires an unresolved external package specifier and a simple
named export. Explicit local exports take precedence; opaque local shadows,
competing stars, cycles, excluded resolved sources and default imports remain
unknown. Static members on the imported value use the same external-call model
as direct imports. Rewrite an ambiguous barrel to an explicit named
re-export, or add an assertion-bearing owner or genuine ownership contract.
Modeled Node I/O through a proven star still requires an assertion owner.

A built-in that invokes a callback keeps the proof: `map`, `filter`, `forEach`,
`flatMap`, `reduce`, `some`, `every`, `find`, `findIndex`, `findLast`, `findLastIndex`, `sort`, `then`,
`catch`, `finally`, `replace` / `replaceAll` (second argument), `Array.from` (second argument), `AsyncLocalStorage.run` /
`exit`, `JSON.parse` (reviver) and `JSON.stringify` (function replacer) in the
second argument on the proven global `JSON`,
and the timers `setTimeout`, `setInterval`, and `setImmediate`. The built-in executes the callback, so an inline function or a
named function the call graph resolves at that exact argument occurrence is
traversed and can reach an assertion. This includes imported helpers, local
aliases, and callback parameters when the canonical call graph proves the
callers' arguments. Direct assertion imports passed as callbacks also count as
executed assertions. Promise continuations (`then` / `finally`) and
`AsyncLocalStorage.run` use the same resolution as array iteration. Imported
aliases of the Node `AsyncLocalStorage` constructor retain receiver provenance.
Proven `run` / `exit` calls also forward their trailing arguments into the
callback for assertion ownership. Unknown or spread arguments retain uncertainty.
Actual built-in invocations also contribute uncertainty to helper parameter
resolution; an unrelated direct caller cannot prove a callback's arguments.

The receiver family must support the callback method: `new Map().then(owner)`
and `(1).map(owner)` supply no ownership evidence. Callback execution is represented
by opt-in `assertion-owner` edges; enabling the rule leaves ordinary `call` queries
and call-site reports unchanged.
Zero-length arrays execute no iteration callbacks. Sparse arrays still execute
the `find` family and `Array.from` callbacks for holes. Mutating or escaping an
array invalidates its length proof and retains dispatch uncertainty. Known empty
results from `slice` and `splice` still execute no callbacks, including on sparse arrays.

An explicit array element type also supplies context for inline callback parameters:
`rows: number[]` makes `rows.map(r => r.toFixed(1)).join(',')` ordinary built-in
work. Declared string properties retain their type through a member read, and a
global `Date.UTC(...)` result is numeric when `Date` is not shadowed. Unknown
elements and stored function elements do not acquire callable identities from an
array annotation.
Both `sort` / `toSorted` comparator parameters receive the element type.
Parameter writes invalidate that context. Proven numeric `toExponential` and
`toPrecision` calls are ordinary built-in work and still require an assertion owner.

An argument with no resolved callable identity (including an unbound parameter,
a conditional expression, a member read, or a call result) remains ``built-in callback `name` is not
statically resolved``. Pass a statically resolvable function or configure a real
ownership contract. A callback occurrence elsewhere in the producer, even one
with the same name, supplies no evidence for this argument. Chained calls retain
each invocation’s argument positions: `Promise.resolve(owner).finally(noop)`
does not execute `owner`, while `Promise.resolve().then(owner).finally(noop)`
reaches it through `then`.
Calls sharing a start offset and spelling must each have their own complete
span proof; an unproven call in that group retains dispatch uncertainty.

```ts
export function owned(items: number[]) { items.map((item) => assertItem(item)); } // reaches assertItem
export function opaque(items: number[], fn: (item: number) => void) { items.map(fn); } // unanalyzable: `fn`
```

Stored callbacks remain dispatch even on proven built-ins. For example,
`const handler = handlers.get('check'); handler?.()` stays uncertain when
`handlers` is a `Map` of callbacks. Likewise,
`checks.forEach(check => check())` stays uncertain when `checks` is an array
of stored functions: the callback parameter `check` has no resolved callable
identity. Storing an assertion-bearing function does not prove its invocation.

## Node `fs` is modeled I/O

Calls into Node `fs` (`fs`, `node:fs`, `fs/promises`, `node:fs/promises`) are
ordinary external I/O: neither assertion evidence nor unresolved dispatch. They
are recognized through named imports, namespace imports, default imports and
their member chains (`fs.readFileSync`, `fs.promises.readFile`), and statically
followed local aliases of those bindings. An `fs` call inside a helper in
another module, and a call through an explicit re-export of a wrapper around
that call, are the same modeled I/O. An `fs` producer that reaches a
configured assertion passes; one that does not gets the usual no-owner finding.

```ts
import { readFileSync } from 'node:fs';
import { equal } from 'node:assert/strict';
export function asserted() { equal(readFileSync('result', 'utf8'), 'x'); } // passes
export function withoutAssertion() { readFileSync('result', 'utf8'); } // no-owner finding
```

Shapes outside the model stay unanalyzable unless an owner is reached elsewhere: computed members
(`fs[name](...)`), dynamic `require('node:fs')`, wrappers the call graph cannot
follow, and a live `fs` call that hands a callback to be invoked later: an
inline callback or promise continuation in the same callable
(`readFile(path).then(cb)`), or a last argument whose syntax may be callable in a callback-style
API such as `fs.readFile(path, done)`, `fs.readFile(path, flag ? first : second)`,
or `fs.readFile(path, first || second)`. Such an argument may be a callback;
its invocation is not a resolved call. For `.call` and literal-array `.apply`,
this classification uses the effective arguments after removing `thisArg` or
expanding the array. `*Sync` and promise APIs
take no callback. The model conservatively treats every other `fs` export as
potentially callback-capable rather than maintaining a callback-signature
allowlist that could omit an API. Scalars, templates without substitutions
and closed literal data containers remain modeled I/O. An object or array is
closed only when all values recursively contain literal data, with static
object keys and ordinary data properties. Literal options such as
`fs.createReadStream(path, { encoding: 'utf8' })` therefore pass when their
result is asserted. TypeScript wrappers preserve this classification.
Functions or classes anywhere inside a container, identifiers, member reads,
calls, conditional/logical expressions, dynamic templates, getters, methods,
shorthand properties, computed keys, spreads and array holes remain uncertain.
The recursive proof also stops conservatively after 16 nested expression
levels. For example, `{ cb: () => {} }` is uncertain, and passes only if another call reaches an owner.
Other modules, such as `node:os`, `node:child_process`, or a PostgreSQL client,
remain unmodeled. A reached assertion owner still passes when the producer also
calls one; with no owner, the call follows `unanalyzable`. Use an ownership
contract for process dispatch that should count as an owner. Tip: set
`unanalyzable: advise` while rolling out the rule to inspect uncertain results
without failing the check.

## Suppression

Rule `include`/`exclude` and project targeting select producers. Dependency
traversal may cross those producer locations to reach actual owners. `message`
customizes diagnostics; uncertainty causes are appended to that custom message. Suppress with `no-mistakes-disable-file
coverage-producer-assertion-owner`, or `no-mistakes-disable-line` /
`no-mistakes-disable-next-line` on the reported producer declaration line.

CLI: `no-mistakes check --root . --format json`. Node: `await check({ root })`,
using the same asynchronous prepared-analysis path and findings/advisories.
`CoverageProducerAssertionOwnerOptions`, `CoverageProducerFunction`,
`CoverageAssertionImport`, and `CoverageAssertionOwnership` are exported types.

## Related rules

[`forbidden-calls`](forbidden-calls.md) restricts configured invocations. This rule instead requires a reachable configured assertion owner for each selected coverage producer.
