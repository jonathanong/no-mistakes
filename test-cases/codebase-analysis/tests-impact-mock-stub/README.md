# tests-impact-mock-stub

Fixture for `tests impact` always-surfacing stub/mock tests.

The vitest suite `exclude`s `**/*.mock.test.mts` (these stubs run in a separate
project), so `widget.mock.test.mts` is normally dropped from test discovery.
`tests.impact.alwaysIncludeTests: ["**/*.mock.test.*"]` opts in to surfacing it
anyway when it transitively imports a changed file.

- `widget.mts` — target component.
- `widget.test.mts` — ordinary suite test (always surfaced).
- `widget.mock.test.mts` — stub test, suite-excluded; surfaced only via the
  always-include glob.
- `helper.mts` — non-test importer; must NOT become a "test" (over-match guard).
