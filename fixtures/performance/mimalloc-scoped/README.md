# Scoped allocator observations

These saved inputs support supplemental measurements for #2200. They cannot
satisfy any full allocator adoption cell. No runtime fixture generation occurs.

`consumer/` is the reviewed #2241 cohort: the unchanged installed consumer at
f38a048f5eeff0438fc59a15e9eb742def575a36, with only its two annotation rule
applications removed (133 to 131). The original sibling `.gitignore` accompanies
the explicit configuration because configuration location affects ignore scope.
The immutable installed archive is shared byte for byte across measurement hosts;
no installed native Node executable is run by these CLI measurements.

`annotation/` scales the existing `postgres-branch-copies` saved benchmark to 32
entry modules, 1,024 literal arguments per call, and 48 conditional joins. The
32 entry files and parameter list were generated once during development and
saved. Arguments retain a cycle and a callback that reads the joined argument;
all 32 unannotated callback queries must remain findings, with no warnings or
advisories. This bounded synthetic project exercises annotation workers and
completes; it does not represent the stalled production annotation project.
The contract records exact input hashes and source bytes. Preflight at fixed main
350ada01a confirmed 32 roots, 1,575,936 incoming dense slots copied, and a completed
annotation phase. Those are semantic checks, not allocator performance results.

Both scopes use fresh CLI processes with diagnostics off/on separately, unchanged
inputs and matched source, compiler, native release profile, and physical host.
Every cell uses four adjacent pairs in AB, BA, BA, AB order. Each platform runs
all job counts on one host, recording actual automatic Rayon workers for max.
Wall time, summed user/system CPU, and wait4 peak RSS include startup and reporting.
Censored samples are preserved and rejected. Four pairs provide exploratory
observations, not an adoption or tail bound; the full gate stays blocked.
