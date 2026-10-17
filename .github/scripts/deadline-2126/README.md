# PR 2126 active-deadline diagnostic (reviewed source)

This package requires the reviewed status, exact published source SHA, and complete
package manifest confirmation. Its source is activated; no execution has occurred.
No build, CLI invocation, publication, or workflow dispatch has occurred in preparation.
It addresses the missing current-head active-timeout measurement, not the old unsafe
99e86587 timings. Existing CodSpeed ARM results cover the inactive deadline path and
report 223 no-detected-regression comparisons; they do not prove an active-path gain.

Two fresh same-run ARM release CLI producers bind exact 2bb75add versus ede95124.
No addon is built. Each producer retains compiler/Cargo JSON (fresh:false), actual CLI
hash/ELF AArch64, complete tracked source and dependency-file inventories, features,
Cargo.lock, machine, and cleanup receipts. Default features include Mermaid; release
uses repository opt3/full LTO/cg1/debug0. No shared target/cache or older artifact fallback.

Six ARM VMs each own one diagnostics × jobs case. They download BOTH artifacts from
this run and check source/run/attempt/compiler/hash/size/ELF identities. Both timeouts
120 and 0 run on that SAME VM with the SAME fixed Git checkout at baseline2bb and saved
empty-rules.json. Git index discovery is preserved: fixture-inventory.json pins all
23,370 Git entries, modes, byte hashes, and tree. The ~23k discovery count is separately
required and equal across the verbose preflights. Git metadata and lock paths are not
invented by a git-archive extraction. Source files become read-only; each process gets
an owned XDG runtime outside the fixture. No dependency install touches this fixture.

Four untimed preflights/case (both arms × both timeout modes) prove successful parsing,
valid empty config, full JSON/exit parity and capacity BEFORE all timed rows. On cases
use --timings --verbose-timings and require actual discovery.classify plus candidates;
off cases do not enable observers. Timeout0 is disabled by InvocationArgs.options ->
nonzero_seconds; both original refs must retain that exact source behavior at review.

Each of twelve cells has ABBA then BAAB, eight process-cold rows; 96 rows total.
OS page caches are warm, and the two timeout modes have a fixed disclosed order.
Default jobs means omitted --jobs, not a claim of 28 workers: record available CPUs,
affinity, raw diagnostics, and actual default policy. Explicit jobs1/4 are independent.
Each cell stays within its VM; never pool raw rows across cases or compare VM levels.

A direct wait4 captures elapsed monotonic wall, user/system CPU and peak RSS, including
startup. It is not a phase-only metric. Raw stderr retains diagnostic phase timings.
Rows have a fixed 125s supervisor cap; the active CLI timeout is120s. Each45min case
budgets16*125s timed +4*125s preflight +200s transport. The actual budget is anchored BEFORE setup: min(job-start+2550s, Python-entry+2460s);
preflight requires2200s still remaining after it completes; otherwise STOP before timing.
No rows/blocks may be reduced, rerun, replaced, or filtered to fit. Always-upload keeps
partial/adverse data; technical failure is distinct from descriptive numerical results.

Only TWO blocks/cell: analysis reports geometric paired ratios and both order results,
not a reliable confidence interval/tail guarantee or an inferred speedup PASS. Retain
flat, adverse and uncertain results, full raw outputs/exit, source postchecks and owned
cleanup. No source production edits, gate weakening, release publication or new PR.

Activation (root only): review exact package pins, source flags/parser, fixture Git
manifest, workflow/actionlint and unit results. Copy package to
.github/scripts/deadline-2126 and mirror workflow at an ALREADY REGISTERED diagnostic
workflow path (e.g. receiver-fact-native.yml); API-verify registration/path and published
commit. Protocol is REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC in this final copy.
Freeze pins and exact destination, normal hooks push the diagnostic branch,
verify public archive/hash, then ONE explicit workflow_dispatch with confirm equal to the final protocol SHA256.
No remote-state polling, workflow_run registration dependency, privileged token, or
release asset use. Root retains existing PR ownership; no readiness from workflow green.

V2 safety closure: first job step anchors Unix time BEFORE checkout/tool setup. Python
entry anchors its42min step before inventory/setup. Commands use the lesser deadline,
reserving150s at job end (upload120+cleanup30) and60s inside the step for cleanup.
Preflight cannot reset either clock. Fixed schedule fit requires2200s still available;
slow setup/preflight STOP before timing, with all raw retained. PASS is written only
AFTER finally verifies fixture/binaries/complete package and no owned children; any
setup, row or finally failure writes FAILED. Approved source SHA must equal both native
GITHUB_SHA and GITHUB_WORKFLOW_SHA. Input-bound complete package manifest verifies every
file (no unexpected files/symlinks) and protocol hash/status remains independently gated.
Local fixed Git readers have30s timeout, hooks and fsmonitor explicitly disabled.
