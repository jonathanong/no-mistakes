# `no-mistakes/vitest-timeout-cap`

## Why

Receiver-only callback `.bind(thisArg)` preserves TestContext parameter positions,
including test.for's second parameter. Pre-bound or spread arguments produce an
unresolved callback-carrier finding at a known registration rather than silently
assuming an unchanged context position. Ordinary unrelated bound callbacks remain
outside framework admission; imported callback implementations are not resolved.

Named callbacks, immutable callback aliases and function declarations acquire
TestContext only through actual collected test registrations. `test.for` uses
its second context parameter; `test.each` data is not TestContext. Hook/test/runtime
`.call` and `.apply` retain their known API identity: call drops the receiver,
apply admits only literal argument arrays without holes or spreads. Opaque arrays
and spread arguments produce unknown findings under finding policy rather than
silently dropping the framework call. Ordinary lookalike calls remain outside
admission. This does not establish cross-file configured deadline/latch closure.

Very long test and hook timeouts can hide hangs and make CI spend minutes
waiting on a test that should fail promptly. Keep normal timeouts small, and
use a specific synchronization signal or test seam when work is legitimately
slow.

## Disallowed

This opt-in ESLint/Oxlint rule reports zero, negative, or oversized effective
Vitest configuration values and per-test, suite, and hook overrides. Zero disables a
Vitest timeout; negative values are invalid. It recognizes imported and aliased
Vitest APIs, `defineConfig`,
`defineProject`, and `mergeConfig` imported from `vitest/config` or `vite`,
project entries, and `vi.setConfig` updates to `testTimeout` or `hookTimeout`.
The default caps are 5,000 ms for configuration `testTimeout` and `hookTimeout`,
and 30,000 ms for individual test, suite, hook, and runtime overrides.

```ts
import { defineConfig } from "vitest/config";
import { it, beforeEach } from "vitest";

export default defineConfig({
  test: { testTimeout: 0, hookTimeout: 45_000 },
});

it("waits too long", { timeout: 60_000 }, async () => {});
beforeEach(async () => {}, 45_000);
```

## Allowed

Positive values at or below the cap are accepted. The rule evaluates same-module
constants, object and array literals, known spreads, and config callbacks with
expression bodies or a single return statement. It applies deep object merges
and concatenates merged arrays, so only effective config/project timeout
values are reported; a known value replaced by a later property or spread does
not produce a finding. Numeric separators are understood. A dynamic config or
spread that cannot be recovered is opaque; for a recognized timeout field
whose value is unresolved, set `unknownValues: "finding"` to report it.

```ts
const shortTimeout = 4_000;

export default defineConfig({
  test: { testTimeout: shortTimeout, hookTimeout: 5_000 },
});

it("uses a bounded timeout", { timeout: 25_000 }, async () => {});
```

Returned object bindings remain known for synchronous expression-body or
single-return callbacks whose immutable aliases are consumed only by
`defineConfig` or `defineProject`. For example, `const settings = { test: {
testTimeout: 1 } }; const make = () => { return settings; }; defineConfig(make)`
is accepted under `unknownValues: "finding"`. Unknown callback consumers,
exports, additional body statements, async/generator callbacks, and parameter
defaults or destructuring retain unknown values.

Module `const` bindings do not make referenced objects immutable. Member writes,
updates, deletion, aliased nested references, and unknown helper escapes retain
unknown timeout values. Strict `unknownValues: "finding"` rejects these carriers;
a later explicit property can restore a known value. Immutable scalar constants
and unmodified object aliases remain supported.

Assignments and updates on shallow copies require a proven own data property.
Own or inherited setters can mutate shared children through `this`, so accessor
writes and missing or unresolved property descriptors retain unknown values.
Object spread creates data properties from source accessors; deleting a copied
property does not invoke a setter. Spread copies enumerable own properties;
array `length` is an own data property on the array but is absent from an object
copy unless explicitly declared there. An inherited or existing setter for
`length` therefore retains unknown values after spreading an array into an object.
For a property named `__proto__`, concise methods, computed names, and shorthand
properties are own data properties. The non-computed colon form changes the
object's prototype, so it does not establish an own data property proof.

Object and array spreads make shallow copies. Replacing, updating, or deleting a
copy's own property does not make the source's timeout unknown; the copy's own
timeout still becomes unknown. Nested references remain shared, so a write such
as `copy.test.testTimeout = 30_001` also makes the source uncertain. Unknown copy
escapes and uncertain alias identities remain conservative.
Known `null` spreads contribute no properties, so `{ ...source, ...null }` and
immutable aliases of `null` retain the copied scalar proof. Mutable null aliases
and unresolved spread values remain conservative.

The primitive member-read shortcut applies only to identifier declarations.
Destructured bindings do not use their container initializer as the bound value;
member reads that escape to an unknown consumer retain unresolved timeout values.
Use a direct `const` object binding when its timeout needs static resolution.

A shallow copy inside an object or array wrapper retains its boundary through
known property paths and immutable aliases: `box.copy.timeout = 30_001` leaves
the source's timeout known. Shared children below that copy remain uncertain.
Erased TypeScript assertions, `satisfies`, and non-null assertions around the
copy or its enclosing wrapper preserve that same property path. Deletion aliases
through those wrappers still invalidate descriptor proofs before assignments
that might expose an inherited setter.
Unknown computed wrapper keys, array offsets after opaque spreads, and custom
iterables retain conservative handling.
Statically sized array spreads, including sparse arrays and immutable array
aliases, preserve known wrapper offsets; custom or mutated iterables remain
conservative.

Public Vitest 5 fixture tuples carry `auto`, `injected`, and `scope` options, not a
fixture `timeout` option. `extend`, `override`, and deprecated `scoped` preserve
registrar identity, so actual test and hook overrides on their returned APIs are
still checked. This rule does not invent a deadline from arbitrary fixture data.

## Options

- `defaultMax` sets the maximum config/project `testTimeout` and `hookTimeout`
  in milliseconds. It defaults to `5000`.
- `overrideMax` sets the maximum timeout for test, suite, hook, and
  `vi.setConfig` overrides. It defaults to `30000`.
- `unknownValues` is `"ignore"` by default. Set it to `"finding"` to report
  unresolved timeout expressions.

Both numeric caps must be positive finite numbers. Set the rule in the files
that contain Vitest tests or configuration:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [
  {
    files: ["**/*.test.*", "vitest.config.*"],
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/vitest-timeout-cap": [
        "error",
        {
          defaultMax: 8_000,
          overrideMax: 20_000,
          unknownValues: "finding",
        },
      ],
    },
  },
];
```

- `configRoot` is `false` by default. Enable it only in explicitly selected
  Vitest configuration files to check a plain standalone default-export object.
  An unrelated default export in an ordinary source file is not admitted.

Immutable local API aliases, renamed destructuring, matching bound runtime
receivers, static computed names, `vitest.setConfig`, current suite chains and
returned `extend`/`override`/`scoped` APIs retain SDK provenance. `aroundAll`,
`aroundEach`, global completion hooks and completion hooks on the actual test
context are checked. Table data is not a TestContext. Partially bound or
unresolved framework calls remain findings under finding policy; later call
arguments are never substituted for pre-bound arguments.

Opaque recognized config/test/project/runtime/options carriers and unresolved
relevant spread/merge keys also produce findings under finding policy, including
carriers whose literal timeout field was never observed. Explicit later
overrides recover relevant fields without guessing imported config contents.
These file-local findings do not claim cross-file helper/project closure.

The rule does not discover tests or inspect files outside the configured
globs. Include project configuration files and any Vitest project files that
should be checked.

## Fix

Use the shortest timeout that covers the work and make asynchronous completion
explicit. Replace arbitrary sleeps with a readiness signal, event, or fixture
barrier. For slow external work, move setup behind a deliberate test seam and
keep the required environment provisioned; do not raise a timeout to mask
missing credentials or a stalled dependency. If a larger limit is justified,
set a documented `defaultMax` or `overrideMax` for the applicable test files.

## Suppression

Prefer fixing the wait or configuring a justified cap. A reviewed exception
can use the rule-specific directive:

```ts
// no-mistakes-disable-next-line vitest-timeout-cap -- exercises the slow migration path
it("runs the migration", { timeout: 45_000 }, async () => {});
```

`no-mistakes-disable-line` suppresses a finding on that line.
`no-mistakes-disable-file` opts out the whole file when placed before the
first code token. Directives accept either `vitest-timeout-cap` or
`no-mistakes/vitest-timeout-cap`. Standard ESLint/Oxlint disable directives
also apply.

## Related rules

- [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md)
  limits Playwright assertion waits.
- [`test-no-skips`](test-no-skips.md) prevents skipped tests from hiding
  incomplete coverage.

Numeric arithmetic (`+`, `-`, `*`, `/`, `%`, `**`) is resolved when both operands
resolve to numbers, including module constants. This catches `60 * 1000`
and accepts a `30 * 1000` override at the cap. Infinity is over cap; NaN
remains unknown. Bitwise operators remain unresolved rather than duration arithmetic. Strings are not coerced and arbitrary calls are not executed.
Unresolved expressions still follow `unknownValues`; use `"finding"` to report
them. This source rule does not inspect shell or Vitest CLI timeout flags.

Replacing or deleting a shallow copy's own data property preserves its source.
Increment, decrement, and arithmetic compound assignments preserve that source
only when the copied property is proven numeric or null. Coercing a shared object
can run `valueOf` or `Symbol.toPrimitive`, so its source stays unknown under
`unknownValues: "finding"`. Logical assignment does not coerce the previous value.
For a copy created directly by an object literal, a replacement after its scalar
update in separate top-level module statements does not invalidate that earlier
update. Prior replacements, aliases, branches, loops, function bodies, nested
blocks, and combined expressions retain the conservative ordering proof.
Type assertions and non-null assertions around the whole copied member preserve
these checks, including rejection after the copied scalar is replaced by an object.
Literal object and array wrappers preserve this scalar proof, including nested
paths such as `box.items[0].copy.counter++`. The wrapper path and copied property
must have known data initializers. Replacing a receiver, exposing its alias,
overwriting the scalar with a non-scalar value, or using uncertain keys, getters,
prototype setters, or opaque spreads keeps coercion unknown. Identifier-backed
wrapper slots and array spread wrappers also stay conservative. Writes that do
not coerce retain their copied path proof.
Recognized test, hook and runtime option consumers remain allowed during
the direct copy's scalar proof. Arbitrary copy escapes and aliases stay conservative;
admitting a copy consumer does not make the mutated copy itself a statically unchanged value.
Configuration helpers can return the original copy, so those calls do not establish
a safe scalar proof through a potentially mutable returned alias.

Object exports (including exported declarations and aliases) and unproved consumer
contexts such as yield, await, tagged templates, and computed property keys retain
unknown values. Exported numeric scalar budgets remain supported. Object stability
uses the existing ESLint binding references and admits only recognized configuration
and registration option positions; it does not prove arbitrary consumer behavior.

Reading a statically resolved numeric or null property does not expose its options
object to the reader. For example, `Math.max(options.timeout, 0)` keeps an unchanged
`const options = { timeout: 1 }` known. Passing an object-valued property to an
unproved consumer, reading a getter, or writing a timeout still keeps analysis
conservative.

Callback confinement also follows identity-preserving `defineConfig` and
`defineProject` results through immutable aliases and nested config helpers.
Exports, unknown consumers, direct invocation, and mutation of those results
leave the returned config unresolved. A standalone helper call preserves the
confinement proof.

Vitest re-exports Vite's `mergeConfig`, which accepts object-form configs. Passing
`defineConfig(make)` as a merge operand still passes a callback and throws. Invoke
the callback first to supply an object, for example
`mergeConfig({}, defineConfig(make)())`. This runtime-valid invocation does not
establish stability of captured bindings: those remain unresolved under the
existing conservative callback proof. Literal object-form merge operands retain
normal timeout analysis; no general detachment of nested references is inferred.

Erased TypeScript `typeof` type queries, including qualified property queries,
do not participate in the runtime stability proof. A callback or config binding
can retain a known timeout when it also appears in a type alias. Runtime
consumers still require the same confinement proof.

Function-valued properties and callable aliases remain object references even when
the configuration evaluator can resolve their return values. Passing such a member
to an unproved consumer retains unknown diagnostics; supported `defineConfig`
callbacks still use their returned configuration. Direct timeout properties also
reject callable values: `{ timeout: () => 1 }` is unknown rather than a numeric
one-millisecond budget, even without an earlier opaque consumer.
