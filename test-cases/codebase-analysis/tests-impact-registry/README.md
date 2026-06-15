# tests-impact-registry

Fixture for the `tests impact` registry hint.

`tests.impact.registries` lists glob patterns for hand-maintained registry files.
When a changed file is imported by a file matching one of these globs, `tests
impact` emits a `registry-hint` warning so the agent verifies the registry entry.

- `feature.mts`, `feature2.mts` — target components.
- `auth-gated-code-splitting.mts` — registry that dynamically imports both
  features (matched by the exact-name glob).
- `widgets-registry.mts` — second registry (matched by `**/*-registry.mts`).
- `plain-consumer.mts` — ordinary importer; must NOT produce a hint (negative).
