# Expected low helper recovery sites

These comments are the saved case labels. For each low policy configuration,
`EXPECT_CLEAR` sites have no finding or advisory, `EXPECT_ADVISE` sites retain
an unanalyzable advisory, and `EXPECT_OFFSET_*` sites produce the no-offset
finding at the SQL token's physical origin. For `EXPECT_OFFSET_HELPER`, that
origin is the marked line in `src/helpers.mts`; for
`EXPECT_OFFSET_CALLER`, it is the caller's literal argument in `src/query.mts`.

`EXPECT_CLEAR_BIND_DATA` specifically protects the library contract that every
`sql-template-strings` interpolation is a bound value. The nested
`SQLStatement` containing `OFFSET` must not be copied into the outer SQL text.
The bound-value behavior applies to each configured low rule.

`src/unsafe-helpers.mts` keeps the branch and work models small. The private
helper-recovery unit harness should use reduced `Limits` for
`EXPECT_ADVISE_VARIANTS_TEST_OVERRIDE` and `EXPECT_ADVISE_WORK_TEST_OVERRIDE`;
those cases exercise fail-closed budget plumbing without turning max fixture
sweeps into large SQL parses. Depth and recursion controls use the ordinary
limits and must retain advice at the executor call site.

The max config is a saved control for existing max behavior. Helper recovery
here is intended to be additive to low effort only.
