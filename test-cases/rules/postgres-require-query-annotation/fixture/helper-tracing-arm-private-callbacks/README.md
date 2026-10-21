The no-deletion case creates the same factory closure in both unknown-condition
arms. Each returned callback must retain its own captured frame and argument
object. Two events at one source occurrence retain different concrete SQL
candidates: one unannotated and one annotated. The opaque consumer may mutate
arguments, so these remain uncertain possibilities rather than complete SQL
projections. Under ignore mode, the known unannotated candidate must still
produce one finding. The deletion case uses separate factories so one arm
unconditionally deletes its own argument slot. This definite deletion is owned only by one arm;
it must not remove the unsafe SQL retained by its sibling.

The configured fixture expects two findings total, at no-deletion.mts:4 and
deletion.mts:4, with identical observer-on/off reports. Evaluator tests
also inspect the actual private identities and deletion mask. A separate test
runs the saved factory arms against independently initialized equal baselines:
their numeric IDs collide despite different SQL. This proves that a future
parallel implementation must translate private namespaces before joining; no
parallel implementation or deliberately incorrect merger is added here.
