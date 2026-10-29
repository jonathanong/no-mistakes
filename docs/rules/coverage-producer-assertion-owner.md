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
imports match resolved module/export identities, including named and namespace
imports and imported aliases. Default-import member calls such as
`import assert from 'node:assert'; assert.strictEqual(...)` remain unanalyzable;
a locally shadowed identifier or a string containing an assertion name does not
qualify. Helpers need no local assertion when their actual call descendants own
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
function execute() { return 1; }
export function produceCoverage() { strictEqual(execute(), 1); }
```

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

`maxDepth` (default 64) and `maxNodes` (default 10000, including the producer)
are positive traversal bounds. A reached bound with unexplored work is
unanalyzable. Dynamic targets, unknown alternatives, source/parse failures, and
unmodeled external module calls also follow shared `unanalyzable` policy:
`report` emits a finding, `advise` emits an advisory, `ignore` explicitly omits
that uncertain result. Uncertainty never supplies assertion evidence, and is
reported even when another reachable call owns an assertion. Known globals
retain the call graph's classification; external process calls need contracts.
Unmodeled external module calls keep the producer unanalyzable even when an
assertion is also reached.

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

Shapes outside the model stay unanalyzable, never passing: computed members
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
levels. For example, `{ cb: () => {} }` is uncertain even when followed by an
assertion.
Other modules, such as `node:os` or `node:child_process`, remain
unmodeled; use an ownership contract for process dispatch. Tip: set
`unanalyzable: advise` while rolling out the rule to inspect uncertain results
without failing the check.

## Suppression

Rule `include`/`exclude` and project targeting select producers. Dependency
traversal may cross those producer locations to reach actual owners. `message`
customizes diagnostics. Suppress with `no-mistakes-disable-file
coverage-producer-assertion-owner`, or `no-mistakes-disable-line` /
`no-mistakes-disable-next-line` on the reported producer declaration line.

CLI: `no-mistakes check --root . --format json`. Node: `await check({ root })`,
using the same asynchronous prepared-analysis path and findings/advisories.
`CoverageProducerAssertionOwnerOptions`, `CoverageProducerFunction`,
`CoverageAssertionImport`, and `CoverageAssertionOwnership` are exported types.

## Related rules

[`forbidden-calls`](forbidden-calls.md) restricts configured invocations. This rule instead requires a reachable configured assertion owner for each selected coverage producer.
