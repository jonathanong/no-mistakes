# PostgreSQL branch-state copy benchmark

Twelve entry modules call one helper with 64 literal arguments. Its two branches
write different unannotated SQL strings to one slot of a cyclic `arguments`
object, leaving the other dense slots unchanged. A returned callback reads that
joined slot for the checked query. Each entry also checks one static annotated
query, so the expected report has twelve findings and no warnings. The helper's
saved `retained` value lets a Rust test assert callback, joined-slot, and cyclic
argument reachability directly.
The fixture is saved on disk and is shared unchanged by baseline and candidate
benchmarks.
