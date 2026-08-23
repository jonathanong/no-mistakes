# `no-banned-import-outside-allowed-paths`

Fixtures for the generic path-scoped capability-ban rule. `invalid.ts` exercises every
tracked binding path (static import, dynamic import, `require`/`createRequire`, namespace
import, aliasing, destructuring, spread, and re-export) for names configured as banned by
the shared test options. `valid.ts` exercises the corresponding non-matches: unrelated
modules/names, shadowed bindings, and binding paths the rule intentionally does not track
(matching the reference `no-global-fetch-outside-helper` rule's documented precision
boundary).
