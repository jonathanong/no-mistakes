Consumer acceptance matrix for repeatable `forbidden-calls` policies.

The packaged `check` CLI loads this fixture's YAML, discovers real Vitest and
Playwright configs, and asserts JSON findings plus clean cases. It is the
release-gating contract for timer, mock-migration, root-exclusion, glob, and
discovery behavior. Root-exclusion cases distinguish filtered seeds from
findings reached through retained roots and rule-level finding filters.

Assertions identify each material source case by application name, YAML
application index, target, import spelling, and line. Absence-only cases pair
with a selected sentinel finding so a skipped root cannot satisfy the check.
