Check that Next.js redirect and rewrite destinations resolve to App Router pages.

`tracked-routes` is copied into a synthetic Git repository by native unit and CLI
tests. Only its tracked/group/dynamic/catch-all pages are staged initially;
`untracked` remains visible and `ignored` is excluded by `.gitignore.fixture`.
The tests stage both missing pages afterward to prove the findings clear.

The three `limit-*-alias` missing/precedence cases and `fail-unresolved-relative`
report incomplete extraction at the entry config. They protect failed mappings
from falling back to baseUrl or less-specific aliases. `unresolved-*` fixtures
protect entry-config suppressions for findings and advisories at either effort.
`fail-delegate-read` saves a non-UTF-8 delegated source file, which resolves in
the inventory but fails the request source store's strict text read.
`fail-delegate-parse` resolves a configured TypeScript `paths` alias to a
truncated module. The read failure uses a relative re-export; both use the entry
config's delegate line for policy-controlled diagnostics.
The `delegate-*` controls cover entry-line, entry-next-line, entry-file, and
malformed-helper file suppressions.
`limit-catchall-paths` preserves the recall-first finding for a bare specifier
matched by a catch-all `paths` mapping with no local target.
`limit-outside-scanned` saves a valid helper then excludes it from one test's
declared file set to prove that a relative hop cannot silently read outside it.
