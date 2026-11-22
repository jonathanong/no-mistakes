# Annotation intrinsic exposure control

`query.mts` is the only file included by the Max annotation rule. It constructs a leading SQL comment and query with `Array.join`. `unrelated.mts` is disconnected from that source and replaces `Array.prototype.join`. The configured `@bench/db` import is intentionally external; no local client module is needed.

The expected output is zero rule findings and one `postgres-require-query-annotation` advisory on `query.mts:4`, target `leading-annotation`, message ending `unresolved-leading-sql`, with no warnings. Ordinary Max annotation checks also leave this array join unresolved without the mutation, so that output alone is not a differential safety proof. The work-count regression separately verifies that the disconnected file retains its global exposure fact even though only the query file receives full annotation syntax.
