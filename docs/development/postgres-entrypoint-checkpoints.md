# PostgreSQL speculative entrypoint checkpoints

The annotation evaluator initializes a root module, prunes dead state, then evaluates syntactically possible independent entrypoints. Each entrypoint must start from the initialized module state. Events and builder identifiers accumulate across these speculative alternatives; mutations from one alternative must not become another alternative's starting state.

A request-local lazy task iterator preserves the historical order: eligible unmodeled calls, then global functions in name order. Ordinary mode considers direct global functions, including private functions. Precise supplied-source mode uses the existing export reachability selector and recursively discovers function values in the same historical wrappers, with function-start deduplication. Name cycles retain their enter/leave timing across iterator yields. Borrowed child iterators bound pending traversal storage by nesting depth rather than registry width.

The iterator looks ahead by at most two tasks. Zero or one task needs no state checkpoint. Two or more tasks capture the historical 17 fields once; the first task starts directly from that state and each later task restores the checkpoint. The first task still rebuilds the derived fresh mapped argument index. Later restores retain the old field order, including the freshness rebuild's position before other restored argument maps. No final restore is added.

| Number of tasks | Historical captures | New captures | Historical restores | New restores | Fresh index rebuilds, both |
| --- | --- | --- | --- | --- | --- |
| 0 | 1 | 0 | 0 | 0 | 0 |
| 1 | 1 | 0 | 1 | 0 | 1 |
| N >= 2 | 1 | 1 | N | N - 1 | N |

These are operation identities of the algorithm. Representative task distribution and runtime benefit require measurement; root counts alone do not establish either. Test-only probes at the frozen old and new capture/restore sites verify the identities independently of diagnostics counters, while comparing semantic state and accumulated events.

Opt-in diagnostics publish `postgres.annotation.run.entrypoints.zero`, `.one`, `.many`, total `.entrypoints`, `.checkpoint_captures`, `.checkpoint_restores`, and `.fresh_index_rebuilds`. The bucket sum equals completed root runs, restores equal executed tasks minus roots with at least one task, and fresh rebuilds equal executed tasks. The observer is acquired once per run. Local scalar count arithmetic still executes with diagnostics disabled; that path adds no counter maps or clocks.

The iterator introduces tradeoffs that controls must exercise. Precise data-only exported containers can allocate a small traversal stack even when no function exists. With one unmodeled call followed by functions, lookahead can construct the export selector before that first call, changing peak overlap. A sole-task planner is dropped before execution. Saved dense data-only and wide-export lookahead fixtures protect these cases alongside mutation, captured closures, mapped arguments, imported modules, and function alias cycles.

This change does not implement write logs or generational arenas. Multiple tasks still retain and restore complete historical state. The public CLI and Node API continue to consume the same prepared facts and return the same findings.
