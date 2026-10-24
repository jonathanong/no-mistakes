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
imports match resolved module/export identities, including imported aliases;
a locally shadowed identifier or a string containing an assertion name does not
qualify. Helpers need no local assertion when their actual call descendants own
one. Actual resolved callback invocations use the canonical call graph. Storing,
registering, or forwarding a callback without invoking it supplies no assertion
evidence; reference-only callback/member edges are excluded from this proof.
Callback inference requires a resolved actual invocation. Different incoming
callback identities which the canonical graph cannot reconcile remain
unanalyzable, even when those known callbacks individually own assertions.

Counterexample:

```ts
import { execute } from './production.mjs';
import { strictEqual } from 'node:assert';
export function produceCoverage() { execute(); }
function neverCalled() { strictEqual(1, 1); }
```

## Valid example

Fix by calling an assertion-bearing owner from the producer, directly or through
resolved helpers:

```ts
export function produceCoverage() { strictEqual(execute(), 1); }
```

Uncalled sibling/nested functions, assertions in other files without a call
path, literal dead conditional/logical arms and loop bodies, and calls after a sequential `return`/`throw`/`break`/`continue` do not
satisfy the producer. A helper which merely calls the producer is not reachable
in the producer's forward call direction. Branch feasibility beyond these
syntactic exclusions and semantic assertion adequacy remain outside this rule.

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
