# Justify above-default Vitest timeouts

`no-mistakes/vitest-timeout-cap` now reports a known numeric override above
`defaultMax` unless it has an adjacent, non-empty, non-placeholder reason.
With the default settings, values up to 5,000 ms need no reason; justified
overrides can use up to 30,000 ms. Project defaults remain capped at 5,000 ms.

Keep ordinary registrations at the project default. For an intentional exception,
name the work immediately before the call:

```ts
import { it } from "vitest";

// no-mistakes-timeout-reason: waits for the real sandbox provisioning request
it("provisions a sandbox", async () => provisionSandbox(), 15_000);
```

The requirement covers numeric arguments, timeout options, hooks, native aliases,
registrations on extended test APIs, suites, and both timeout fields in
`vi.setConfig`. It introduces no Vitest option. Shared options can carry a reason
immediately before their effective timeout property; a suite comment does not
justify its children. Blank lines or intervening code break adjacency.

`overrideMax` remains a hard cap. A comment cannot permit zero, negative, or
oversized timeouts or higher configuration defaults. `unknownValues` and
`configUnknownValues` retain their existing behavior; a reason cannot turn an
unresolved timeout into a known one. Existing suppression directives still work.

This rule is implemented by the shared ESLint/Oxlint plugin, with no separate
CLI or N-API rule. See the [rule reference](../eslint-rules/vitest-timeout-cap.md)
for accepted comment placement, rejected placeholders, and options.
