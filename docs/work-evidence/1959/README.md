# Nested imported receiver writes: work evidence

Issue: <https://github.com/jonathanong/no-mistakes/issues/1959>.
Source base: `917c9ea1c2f2400e8a5d3d50c18c0a52fe21ad4a`.

The saved fixture is
`test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-writes.mts`.
Its graph test checks member precision, another imported receiver, method
snapshots, replaced ancestor references, mutable aliases, and acquisition timing.
The extraction fixtures also check canonical history sharing and bounded work.

## Baseline probes

The authoritative graph probe used the existing `receiver-current-targets`
harness with the fixture root and `src/nested-receiver-writes.mts` arguments.
Its SHA-256 is
`1fa11a4991c27a7de8637faece3842e23e17285c64f05a8409dead6046ab4646`.
The harness was built from `5b8710b4e0b044fd35b3c770332131ccb857e1b4`;
its production source and Cargo.lock were verified equal to the source base.
[Build metadata](baseline-build.json) and the complete, unmodified
[graph output](baseline-targets.txt) are retained here.

An additional existing CLI probe ran:

```sh
no-mistakes check --root <fixture-root> --format json
```

Its exit status was 1, and its SHA-256 was
`c9bf1601755eaef2addd6f521459019b39c5e66d41f37cf47fa2703816108ad0`.
The CLI's exact source revision is not established; it is supporting evidence,
not an authoritative revision comparison. Its complete, unmodified
[JSON output](baseline-cli.json) includes findings at lines
8, 12, 16, 18, 20, 22, 41, 47, 55, 64, 73, 81, 89, 98, and 107 of the saved file.

## Conservative context handling

The verified baseline reports `primary.get` for all three controls below.
The candidate tests require `Unknown`; candidate execution remains pending.

| Helper | Loop line | Reason for the conservative result |
| --- | ---: | --- |
| `calledReceiver` | 89 | A different callable replaces the ancestor reference before the nested write. |
| `mixedReceiver` | 98 | The replacement summary combines writes from different callable contexts. |
| `inverseReceiver` | 107 | The nested write precedes the replacement callable's invocation. |

The existing facts do not establish execution order across callable contexts.
Only an ordered replacement in the same context can sever the initializer's
receiver path. With uncertain ordering, the candidate retains the possible
mutation of the original receiver instead of claiming its imported method is
unchanged. This deliberately loses the baseline's imported-effect proof for
the first two controls; the inverse control prevents stale provenance after an
earlier mutation. This is not general heap or call execution modeling.

## Validation status

Local compilation is paused for coordinated performance work. Semantic tests,
all 70 existing receiver/default/inline controls, the full graph suite, Rust lint checks,
coverage, complexity checks, and matched performance measurements remain
required before the draft is ready. Work-count assertions demonstrate bounded
sharing, not a measured CPU or memory improvement. Introduced regressions and
material performance regressions block readiness.

Before draft publication, AST rule tests (17), the AST scan, changed Rust
format checks, JavaScript formatting/lint, documentation spelling, and the
repository's configured checks passed. The repository check used the supplied
prebuilt CLI and returned no findings, warnings, or advisories; it did not
execute the candidate Rust implementation. Duplicate-source checks and the
complete pre-push Rust checks remain pending.

## Shepherd Journal

- The initial exact-write propagation consumed the assigned member at the
  syntactic receiver. Retaining it for nested identities fixes the issue's
  path, while ancestor-reference cutoffs protect replaced references and
  previously acquired receivers.
- Static review identified inverse callable execution order as a stale-target
  risk. The candidate keeps uncertain mutation evidence; the precision loss
  above is explicit and must be reviewed before readiness.
- Repeated ancestor checks originally walked the alias component for each
  write. A request-owned union-by-size index now keeps one mutable history per
  component and produces shared final histories. Saved operation-count tests
  cover aliases, repeated writes, and unrelated bindings.

## First draft CI and review corrections

At `b1f2cc155d32110df3d10156f24f41669b3af76f`, CI passed formatting,
Rust lint, duplication, spelling, AST checks, JavaScript, and both native
platform jobs. The Rust library run passed 11,243 tests and failed two existing
default-origin tests: an untouched literal-star selection lost its proof, and
copied-method/replaced-container controls lost findings at lines 16 and 19.
Those expectations remain unchanged.

The correction separates newly propagated exact assignments from direct
assignments inside the existing optional assignment box. Supplied receiver
reads use the aggregate summary. Selected default reads retain their Source
prefix evidence and use their owned writer's propagated summary, preserving
acquisition offsets without importing unrelated helper writers. This adds
optional writer metadata; its cold memory cost still requires measurement.

The conditional-reference review regression is saved in
`nested-receiver-cutoffs.mts`. Its [raw verified baseline output](conditional-baseline-targets.txt)
retains `primary.get` for every helper. Candidate tests require Unknown except
for the guaranteed fresh replacement control. Conditional branches, loops,
short-circuit writes, logical assignments, default right-hand sides,
self-assignment, and unknown values cannot establish a fresh receiver identity.
The earliest guaranteed fresh replacement offset is stored separately from
the earliest possible write. A saved receiver acquired between a conditional
write and a later guaranteed replacement retains its earlier identity.

The traversal counter now measures actual existing alias-edge propagation:
128 writes over 128 undirected links assert 32,768 directed edge visits. That
baseline traversal is still proportional to writes times alias edges. The
canonical root counter separately bounds the additional ancestor checks; only
their redundant component walk was eliminated. No linear whole-collector or
measured performance claim is made. These corrections await current-head CI.

The correction's next CI run passed 11,247 library tests, including all existing
receiver/default graph and rule controls, but exposed two fact guards: the
unchanged 16-byte exact-summary bound and a binding-default traversal bypass.
The next correction packs module/single/mixed writer state into one enum and
stores the guaranteed nonzero completion offset inline, preserving the bound.
Both object and array binding-default paths now retain receiver-only conditional
certainty in the existing named-default visitor. No semantic expectations were
relaxed. Current-head runtime validation remains pending.

The `535349c` run passed Rust tests, documentation tests, and coverage generation; its
weighted CRAP gate failed. GitHub's job-log endpoint returned `Transport closed`,
so the exact failing function report was unavailable. The configured threshold
is unchanged. Root ran the static workspace cargo-crap pass on the revised
working source (`--missing optimistic --no-cache --jobs 1 --min 30`), which passed;
this does not establish the coverage-weighted gate.

Further review identified caught-throw replacement bypasses, later static or
dynamic restoration of the original receiver, and exponential acquisition-route
identities through converging defaults. The next correction retains optional
shared indexed overwrite timelines only for prepared nested-reference components.
Last relevant events are queried at the segment's actual acquisition offset;
conditional fresh writes preserve, but cannot establish, a distinct receiver.
Untracked past writes make a late-marked or merged component incomplete, so its
missing events cannot prove replacement. Ordinary direct-write controls assert
that no event index is allocated. Saved controls retain two static events and
one dynamic event where needed; extra retained metadata and its CPU/RSS cost
remain unmeasured.

Converging paths now join per-segment cutoffs by receiver/path: same-context reads
keep the earliest offset, differing contexts forbid a replacement proof. Only
changed joined summaries propagate, and active cycles cannot mark a summary visited before it propagates. A stored 12-level branching-default fixture counts actual path
visits with a 104-visit bound; it has not yet executed on the candidate. The
existing alias-link bound remains explicit rather than claiming a linear whole
collector. Root statically reviewed the timeline, completeness joins, writer
checks, and returned joined reads before publication. Fresh semantic tests,
weighted CRAP, coverage floors, and performance validation remain pending.

The next review correction marks labeled bodies receiver-conditional because a
labeled break can bypass replacement. Fresh RHS classification now admits plain
and logical assignments separately from guaranteed execution: fresh logical
writes preserve a prior distinct receiver but cannot establish one. Other
compound operators remain conservative. Saved controls cover the labeled bypass,
an ordinary fresh positive, all three logical operators preserving prior
freshness, and all three operators with unknown original identity. Existing
assertions remain unchanged; new positive lines are added explicitly. The
unchanged new-expression visitor body is delegated to keep the 200-line file
limit without changing traversal order. No compiler or weighted-gate claim.


A further review correction makes receiver overwrite certainty relative to the
actual deferred function/arrow body. Guards around callable creation no longer
make every write inside that callable conditional; guards and defaults inside
the callable still do. Generic import-execution facts keep their original flags,
and the Annex B for-in initializer restores both outer execution dimensions.
The saved deferred fixture covers functions, arrows, object methods, labels,
conditionals, try blocks, logical creation, and inner conditional/default
negatives. The fresh-RHS flag is restored before traversing assignment targets
and RHS bodies, so it does not leak from an enclosing assignment into deferred
updates; no extra fresh-state mechanism is added.

The isolated instrumented build at 8eec845 was stopped below the 500 MiB disk
floor before the library test harness completed. Diagnostic evidence was retained
privately; only the abandoned private generated target was removed. This
provided no local semantic or coverage pass. CI retains the unchanged weighted
CRAP command and failure status while publishing its text report and LCOV as a
small diagnostic artifact, because the connected large-job log route fails.


Main 2b04cef6f introduced real content conflicts through #1992 destructuring
write-phase tracking. The required integration preserves its phase receipts,
catalogs, early-read invalidation, instrumentation, and tests alongside nested
receiver summaries. Canonical components normalize early assignments once before
sharing their histories; borrowed component lookups replace upstream map access
without adding per-read name allocation or another alias traversal. This is a
conflict resolution, with fresh combined semantic CI still required.


The next review correction marks non-static class field and accessor initializer
expressions receiver-conditional. Merely defining a class cannot establish a
fresh-reference cutoff. Static initializers and computed keys retain eager
certainty; decorators and computed keys remain outside the new wrapper. The
named-instance-field custom walker uses the same initializer helper. Saved facts
cover anonymous/named instances, accessors, static counterparts and computed
keys. Graph/rule controls require Unknown after the two unconstructed anonymous
instance cases and retain static imported targets at lines 268 and 275. The
verified 917 baseline graph proves primary.get in all four cases; its raw output
is preserved in `class-initializer-baseline.txt` using the same frozen graph
provenance above. The new nested-write semantics require those instance cases
to become Unknown rather than retain a stale imported target. Candidate semantic
validation is pending; no class execution model or generic import-fact flags
are changed.


The ed1d14d run passed semantic tests, documentation tests, coverage generation and lint checks.
The diagnostic artifact identifies one weighted-gate offender:
ValueAssignedMutations::merge, CC 9, coverage 0%, CRAP 90. The unchanged threshold
is 30. Its text summary/provenance is saved in `weighted-crap-ed1-summary.txt`;
the full report and LCOV remain in CI artifact 11932317069. The ZIP hash matches GitHub's artifact
digest. The defensive populated-component merge path was not exercised by prior
late-union controls, which contained only prefix writes.

A saved late-union fixture now supplies real extracted exact histories for
populated components. Its test checks retained restoration/dynamic evidence,
disjoint properties, early destructuring invalidation and aggregate/writer
propagation when those components are joined. This tests the defensive contract
instead of changing the gate or rearranging complexity. Root reviewed the test
and instance-initializer correction; fresh CI remains required.


After all 30 jobs passed at 663f80e, main 44c67f4a7 introduced a genuine content
conflict in receiver mutation collection through #2003. The required merge keeps
canonical component history/proof lookup and compact writer/freshness summaries,
while retaining main's has_unassigned coarse-write flag, its merge OR, and the
projected Source guard. All upstream consumer/default/phase/cache tests and
fixtures are retained. Root reviewed the narrow resolution before publication.
The earlier 663f80e quality results do not validate this combined head; fresh
semantic, coverage, quality and review gates remain required. No unrelated
freshness merge or local compiler is used.


A later review identified an unsafe ancestor cutoff: replacing a wrapper with a
fresh object or array does not prove that its descendant receiver changed. The
wrapper can retain the original client. The correction admits a guaranteed
replacement cutoff only at the final acquired-reference segment; traversal still
borrows the same exact-history trie and recorded acquisition offsets. No new
heap tracker, AST pass, alias inventory, history copy or per-write graph walk is
added. A direct cutoff assertion uses extracted saved-fixture histories to reject
object/array ancestor cutoffs while retaining explicit terminal replacement.

The six separate `nested-receiver-containers/*.mts` controls require original-receiver
invalidation for retained object/array descendants and a wrapper acquired before
replacement. Explicit terminal replacement, a method copied before replacement,
and an unrelated sibling write retain imported get proofs. Existing assertions
remain unchanged. The verified frozen 917 graph reports primary.get for all six
controls, each in an independent source file so earlier mutations cannot contaminate
positive controls. A readable copy of the raw output, its original SHA-256 and
explicit path-normalization label are preserved in `retained-container-baseline.txt`.
No internal workspace or cache paths occur in that copy. The first,
second and acquired-wrapper cases are therefore part of the original nested-alias
precision gap on that baseline, and unsound within this PR's intended mutation
behavior. No claim is made that the new expectations passed on the frozen base.
More general replacement-RHS shape/heap tracking is not implemented. Candidate
semantic and quality validation is pending after this correction.


The guarded-arm correction records a lexical token only for consecutive standalone
plain assignments inside an unknown-if block arm. Exact assignment-start equality
prevents nested RHS writes from establishing that token's proof. Other statements
split the segment; conditional/default/deferred walkers invalidate the token.
Any nested write with a different assignment start also invalidates it, including writes nested in an admitted
assignment RHS, so a captured old LHS cannot prove the subsequently rebound root. The
normal block and statement visitor retains scope, hoisting and binding hooks.
Only prepared nested-reference histories allocate an optional regional event
index. A query uses the current region, actual acquisition offset and same writer,
and rejects later possible-original or dynamic restoration. Global guaranteed
replacement evidence remains unchanged; terminal-only cutoffs remain required.

Saved independent controls cover same-arm replacement, RHS root rebinding, opposite/after arms,
nested children/RHS, logical assignment, restoration, foreign writers and retained
descendants. The frozen 917 outputs in `guarded-region-baseline.txt` resolve all
twelve to primary.get: the same-arm positive repairs introduced precision loss;
negative cases retain the explicit conservative boundaries of nested mutation
tracking. No general execution-path or heap model is claimed. A saved 128-write
control checks one regional index, 128 retained events, 128 record attempts and
128 actual indexed queries; the unguarded control allocates no regional map.
These are work/storage bounds, not measured CPU or RSS improvements. Semantic,
coverage and quality validation of this correction is pending.


The first regional-proof run at 975315b compiled and passed 11,280 library tests,
but failed one graph aggregate assertion: guardedOpposite retained primary.get.
The isolated regional-index and 128-write/cold-map controls passed. Investigation
found a fixture error: generated `let box = {client}` declarations bypass the
existing value-binding collector, which records immutable values and mutable
factory results but does not record mutable object initializers. The original
reviewed regression uses a supported immutable holder. The graph controls now
use `const` holders; saved mutable root-rebind sources remain explicit unsupported
boundary probes, without assertions that their stale Known target is desirable.
Direct extracted facts verify same-arm regional cutoff versus no global cutoff,
and actual imported-member mutation histories for opposite/after/child arms.

Supported immutable-holder RHS ancestor-rebinding controls additionally require
Unknown and one actual admission break: the first LHS captures the old container,
while a nested assignment replaces its ancestor before the next member write.
Every nested write whose assignment start differs from the admitted statement
invalidates the transient token. This covers member ancestors as well as root
bindings without adding RHS heap tracking. Existing expectations remain intact;
new semantic and quality validation is pending after this correction.

The pre-existing mutable object-initializer boundary is tracked separately in
[issue #2019](https://github.com/jonathanong/no-mistakes/issues/2019), queued until
this PR merges. Its exact failed975 graph evidence and frozen917 comparison are
distinguished from the corrected supported immutable-holder controls.


The global acquired-reference correction additionally validates every proper
ancestor at its captured LHS acquisition and the later write acquisition. It
borrows indexed exact histories; fresh terminal evidence alone cannot identify
a replaced ancestor. Captured acquisition paths share immutable suffixes, with
iterative comparison and release. Unknown calls during fresh evaluation and
unproven cross-context identities remain conservative. No general getter, heap
or call-execution model is claimed.

Hot destructuring proof queries reuse the existing phase walk and catalog,
including authoritative target scope receipts. Missing nested-write receipts
remain barriers. An indexed effect stream preserves writes before opaque effects
and rejects proofs afterward. The supported inert-RHS gate rejects calls,
getters, spreads and unproven nested patterns without replaying RHS execution.
A direct own-data read exemption requires one exact prepared object binding, its
actual callable owner, no root rebinding, a compatible execution region and no
unknown call since binding acquisition. Its scalar records calls only after
callee and argument evaluation. Deferred callable/field boundaries suspend it.

The independent saved graph controls and exact input hashes are listed in
[global-identity-917-baseline.txt](global-identity-917-baseline.txt). The frozen
baseline resolves the new negative cases to primary.get, exposing its existing
nested-write gap. Candidate expectations cover ancestor replacement inside RHS,
sequential replacement, saved old wrappers, direct aliases, dynamic paths,
destructuring order, opaque effects and descriptor-changing calls. Pure
before-effect and argument-evaluation controls retain primary.get. These are
expected results; candidate runtime validation is still pending.

Saved production-extraction guards require 32 shared capture nodes for 528
logical acquisition tokens along an alias chain; 16 nodes and 2,063 actual history
visits for 64 deep writes; and 40 phase indexes, 42 RHS classifier visits,
80 trie nodes and 40 indexed write-path tokens for nested assignments. Direct
writes retain no capture or phase index. These assertions are pending remote
execution. Shared nodes and indexes have allocation and metadata costs; operation
counts do not establish CPU, latency or RSS improvement. Earlier CI/performance
results in this file remain historical and are not carried to this runtime.

The first compiled identity correction at25ca1cd passed11,290 library tests,
including the saved capture-sharing, cold-index, nested-RHS work bounds and deep
capture comparison/drop guards. One aggregate graph test failed at the existing
global-phase-preserved-fresh positive: a conditional fresh replacement preserved
detachment, but the new terminal phase query treated it as a possible restoration.
The Known expectation remains unchanged. Quality/coverage gates did not run after
that semantic failure. Run38667260591/job116201941392 records this result.

The focused correction keeps every phase write for proper-ancestor identity, and
indexes possible-original writes separately for terminal detachment queries.
The existing freshness classifier runs only during the already prepared hot phase
walk. Its optional fresh-phase set is consumed during index construction and then
dropped; no per-read replay or scan is added. PhaseWrite retains its prior field
layout. PhaseWalk has an additional optional set header on its transient stack
state, including cold walks, but allocates its set only for hot fresh writes.
Each hot trie node adds one empty BTreeSet header and at most one additional entry
per non-fresh static write. Actual retained-entry and temporary-classification
counters, layout comparison, cold controls, and paired fresh/original restoration
fixtures require fresh remote execution. No measured CPU/RSS conclusion is claimed.

Atf3214b4, Rust job116207426220 in run38668974075 passed library semantics,
documentation tests, weighted CRAP, complexity and all coverage floors. This validates the
34 identity graph controls and direct retained-entry/layout/cold assertions at
that revision. Its lint failure is limited to the nested proof-event argument
type; a named ReferencePhaseEvents owner replaces that tuple without changing
its meaning or lifetime. This passing semantic result does not validate later
changes or establish a performance comparison.

Review4278228393 identified an intervening opaque call omitted after the active
fresh-assignment traversal ended. The correction retains the existing indexed
call stream after the first actual capture, qualified by callable and execution
epoch. Existing post-call/new/tagged hooks also record completion after argument
evaluation, covering calls whose arguments create the first capture. The earlier
pre-walk recording remains for active fresh RHS evaluation; equal end offsets
deduplicate in the shared set. Cold files with no captures allocate no call index.
Hot retention can grow with all calls after the first capture; this is an added
cost, not a speed or memory improvement claim.

Six isolated saved controls cover ordinary/new/tagged calls, argument-created
capture, calls before the holder exists, and deferred callable boundaries.
The authoritative frozen917 baseline resolves all six to primary.get; readable
output and exact fixture hashes are in post-capture-calls-917-baseline.txt.
Prior6ad source also lacks an intervening-call guard; review9b405f1 reports the
candidate's stale target. No independent local candidate execution is claimed.
The direct imported-history, shared call-index, deduplicated entry and cold
controls need fresh remote validation. General standalone getters/coercions
outside existing prepared phase barriers remain an unmodeled boundary; this is
not a heap, escape, callee-purity or general JavaScript effects analysis.
