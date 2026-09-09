Consumer acceptance matrix for repeatable `forbidden-calls` policies.

The packaged `check` CLI loads this fixture's YAML, discovers real Vitest and
Playwright configs, and asserts JSON findings plus clean cases. It is the
release-gating contract for timer, mock-migration, glob, and discovery behavior.

Assertions identify each material source case by application name, YAML
application index, target, import spelling, and line. Absence-only cases pair
with a selected sentinel finding so a skipped root cannot satisfy the check.
