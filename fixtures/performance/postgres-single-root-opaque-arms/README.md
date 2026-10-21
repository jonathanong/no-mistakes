# Single-root opaque callback alternatives

Three saved modules expose one annotation root. Module initialization joins
scalar and callback payloads before an opaque consumer. The callbacks capture a
cyclic `arguments` object and select sibling alternatives. The report has one
finding at `entry.mts:12`; the annotated query remains accepted.

This isolates the evaluator path seen in the consumer profile: module
initialization, callback snapshots, alternative evaluation and whole-environment
joins. It is a small work-count and findings control, not a reproduction of the
consumer's wall time or worker imbalance. Verify the actual root counter: a rule
include filter alone can narrow reported findings without narrowing evaluation.

The companion rule fixture `helper-tracing-arm-private-callbacks` protects the
parallelism boundary. Fresh evaluators starting from equal pre-branch states
reuse numeric captured-frame and argument-object IDs for different SQL. Serial
evaluation instead retains distinct private identities and deletion masks.
Running arms independently therefore requires deterministic translation of both
namespaces before joining returned closures, events, captures and heap state.
Lazy module initialization also creates shared state; its objects cannot all be
treated as arm-private. These tests do not implement such a translation or claim
that copying a complete evaluator per arm would be affordable.
