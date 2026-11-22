# Annotation fact scale workload

The saved workload has 64 entry modules, 4 shared callback modules, a shared payload module with a depth-16 scalar record DAG, and 4 excluded unrelated modules. Each entry passes the same imported payload through the callback chain, where it is unused, and calls a shared callback containing an explicitly annotated query. It also has one direct unannotated query marked `expected-finding`.

The exact report contract was confirmed on release `v0.109.109` at both effort levels: 64 expected findings, 64 proven-clear SQL verdicts, and zero abstentions, advisories, or warnings. `expected.json` records this preflight and the saved source inventory: 73 source files, 557 physical lines, and 21137 bytes.

The deterministic CI ceiling is the work-count assertions in `codebase::postgres::query_annotation::evaluate::effects::call_value_work_tests`: scalar-record refresh visits are zero when there are no builder tables, at most one with an unrelated builder invalidated, and at most one for the callback scalar-record case. These count assertions guard the repeated argument-refresh cost. Wall-time max/low ratios are diagnostic only; timings vary with host load and are not a CI gate.

Regenerate the saved sources and configs with `python3 materialize.py` from this directory. Optional `--roots N --depth N` parameters let profiling vary scale; regenerate the seed defaults after experiments before committing. Generated TypeScript is saved for review, and benchmark runs read the corpus in place.
