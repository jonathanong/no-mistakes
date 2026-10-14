# `no-mistakes/playwright-test-timeout-cap`

## Why

Keep test and hook deadlines positive, finite and bounded. This opt-in ESLint/Oxlint
rule owns Playwright test deadlines; `playwright-assertion-timeout-cap` continues
to own assertion waits separately.

## Disallowed

The rule checks admitted `test.setTimeout`, actual test/hook callback TestInfo
setters, `test.info()` setters, `describe.configure` timeout fields, recognized
configuration/project timeout fields and fixture tuple timeout options. It follows
immutable aliases, renamed destructuring, namespace imports, bound setters with
matching receivers and statically known computed names. Partially applied setters
or slow methods remain unresolved findings; their pre-bound arguments are never
replaced by later call arguments. Dynamic members on a
proven framework API are opaque findings under `unknownValues: "finding"`.

```ts
import { test } from '@playwright/test';
const { setTimeout: deadline } = test;
deadline(0);
test('too slow', async ({}, info) => {
  info.setTimeout(15000);
  info.slow(); // Effective45000 exceeds the default30000 cap.
});
```

## Allowed

The following describes locally legal operations, not full inherited-slot proof.

Positive finite values at or below the cap are accepted. Numeric arithmetic uses
an existing value evaluator, not execution or coercion. False slow conditions are
inert. A proven5000 slot becomes15000 after one slow call; repeated slow calls do
not multiply again. A later setter replaces the slot without resetting the SDK
slow latch. Ordinary fixture payload `{timeout:60000}` is not a fixture deadline;
only the SDK's fixture tuple options carry that budget.

```ts
import { test } from '@playwright/test';
test('bounded', async ({}, info) => {
  info.setTimeout(5000);
  info.slow();
  info.slow(); // Still15000.
});
```

These local setter/slow operations are legal. Under finding policy the test
registration still receives `effectiveSlot`: the preceding inherited deadline and
latch have not been proved by this callback. This is not a clean-file example or
an assertion that the legal slow operations exceed the cap.

## Options

- `max`: positive finite test/suite/project/runtime cap, default30000.
- `fixtureMax`: separate positive finite fixture cap, defaults to `max`.
- `unknownValues`: `"ignore"` by default; `"finding"` reports unresolved admitted
  values, config/project/fixture carriers and effective slow slots/conditions.
  It also reports `effectiveSlot` at each admitted test, chained or hook
  registration whose inherited configured deadline/latch lacks owner proof.
- `registrationPackages`: explicitly approved module names exporting Playwright
  `test`, `mergeTests` or `defineConfig`; defaults to only `@playwright/test`.
  This is an admission boundary, not proof of an external helper's implementation.
  Unknown wrapper exports from an admitted module produce unresolved findings.
- `exportRoles`: module-specifier-to-export-name maps with roles `"assertion"`,
  `"ordinary"` or `"opaque"`. For example, after inspecting a helper that re-exports
  SDK expect, `{ "../../helpers/test.mts": { "expect": "assertion" } }` keeps its
  assertion calls outside deadline-registrar classification. Repeat for actual
  literal import spellings. Unclassified helper exports remain opaque. This is
  explicitly reviewed producer provenance, not automatic export resolution.
  Never label an unresolved registrar ordinary to waive a finding. Nonempty maps
  for `@playwright/test` are rejected; `test`, `defineConfig` and `mergeTests` roles
  cannot be overridden in any module. Known SDK/registrar identity is immutable.
- `configFiles`: exact filenames or path suffixes whose plain default export is
  a Playwright config root. An unrelated default-export object is not a config.

Known later object fields overwrite earlier fields. Unknown spreads preserve
uncertainty unless a relevant field is explicitly replaced later. Config timeout
absence is not itself an invalid or disabled deadline.

## Scope and unresolved evidence

SDK `import { default as test }` has the same builtin identity as a default
import; arbitrary helper defaults do not. Callback `.bind(thisArg)` preserves
parameter positions. Pre-bound arguments or spread binding arguments instead
produce an unresolved bound-callback carrier finding at the actual registration;
the rule does not pretend shifted parameters are still TestInfo. Immutable aliases
of these callbacks preserve that distinction. Receiver-only bound callbacks retain
locally legal false/short slow behavior, independently of inherited-slot findings.

The public default import and namespace `.default` identify test only for
`@playwright/test`, never arbitrary helper defaults. Named callbacks, const callback
aliases and function declarations acquire TestInfo only through a collected
registration using that function. Similar parameter names on ordinary functions
do not supply framework provenance. Multiple registration owners do not prove a
single initial slot/latch; inherited effectiveSlot findings remain.

This file-local rule does not resolve imported config values or build a new
cross-file parser/graph. Such carriers produce findings under finding policy, including opaque registrar
arguments to `mergeTests`, unresolved approved helper wrappers and imported
fixture entries whose tuple/options shape is unknown.
Straight-line setters inside a proven test/hook callback establish its local slot
for subsequent slow calls. When an inherited latch is unresolved, multiplication
is only an upper bound: a legal upper bound is accepted, while an over-cap upper
bound produces an unknown finding rather than falsely claiming multiplication; unresolved inherited project/suite slots, conditional
control flow and fixture-callback slow conditions remain unknown. These source
findings do not claim full project/helper registration closure. Prepared upstream
integration facts are required for that separate inventory/closure proof.

Bare registrations do not silently pass inherited-slot uncertainty. Finding
policy emits `effectiveSlot` even when subsequent local setters are bounded;
these do not prove the deadline active before the setter. Unknown methods on an
admitted test owner also produce carrier findings. Known SDK info/step roles are
separate from test-slot setters; this rule does not qualify step deadlines.
Assertion roles retain the separate assertion rule. Ordinary calls may invalidate
local slot/latch knowledge because their effects are not proved.

The current implementation supplies unresolved findings, not cross-file owner
proof. Longer-term closure belongs in existing canonical prepared owner facts,
with fixtures for effective project/suite/helper slots and latch provenance.
There is no unchecked proof option or alternate consumer parser. Factory-wrapped
config outputs remain findings until their actual producer values are resolved;
an opaque finding does not repair an over-cap consumer deadline.

## Fix

Use explicit bounded deadlines and synchronization signals. Keep imported/helper
ownership explicit and unknown admitted carriers visible until their owning
prepared config facts can resolve them. Do not replace uncertainty with a larger
cap or automatically reject a proven legal short slow slot.

## Suppression

The normal `no-mistakes-disable-file`, `no-mistakes-disable-line` and
`no-mistakes-disable-next-line` directives apply with this rule name. Suppressions
are deliberate source scope, not a substitute for resolving unintended opacity.

```ts
import { test } from '@playwright/test';
// no-mistakes-disable-next-line playwright-test-timeout-cap -- intentional unresolved owner fixture
test('owner proof example', async () => {});
```

## Related rules

- [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md) owns assertion waits.
- [`vitest-timeout-cap`](vitest-timeout-cap.md) owns Vitest timeout carriers.

Callable timeout values remain unknown even when their return expression is a
known number. For example, `test.setTimeout(() => 1)` does not establish a
one-millisecond deadline, and a subsequent `test.slow()` retains an unknown
effective deadline. Numeric timeout values retain their normal cap checks.

## Delegating unresolved configuration values

`configUnknownValues` defaults to `"inherit"`, preserving `unknownValues` behavior.
Explicit `"defer-to-runner"` delegates only unresolved configuration/project
carriers to a separate effective-runner admission policy. Known invalid and
oversized numeric configuration values still report. This option does not change
case, hook, fixture, registration, suite or runtime-setter analysis; inherited
Playwright slot findings remain findings.

This option is a consumer policy decision, not proof that an external guard exists.
Use it only where the normal runner continuously validates effective selected
project budgets, including CLI overrides, before test execution, with required CI
controls that fail when that owner is absent. A clean baseline snapshot, alternate
config bypass, filename exclusion or general unknown-value ignore is not a
substitute. The option does not establish repository-wide timeout closure.

## Fixture tuple positions

Fixture tuples follow Playwright's runtime predicate: an array whose actual
second slot is an object. Extra tail elements do not hide timeout options, and
array holes retain their runtime positions. A spread with unknown length before
that slot remains an unknown carrier; an unknown tail after a proved second slot
does not move its options. Mutated array identities lose their positional proof.

Proved primitive or function second slots are ordinary fixture data, including a
function whose projected return value is an object. Null and unresolved
object-like options remain findings under `unknownValues: "finding"`; this rule
does not claim that an SDK-invalid null options value is a valid fixture.
