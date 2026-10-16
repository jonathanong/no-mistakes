# Isolated x86 diagnostic source proposal

This source-approved adapter reuses the four closed c896/099-source-equivalent x86 CLI/addon
ELFs. It builds no analyzer, changes no production PR, and reuses no old rows.
The original 97-file local freeze and canceled R1 remain untouched. The new host
is an isolated GitHub `ubuntu-24.04` x86 VM with CPUs 0–3 and Rayon 4. This is
separate hosted evidence, with no ARM, Criterion, or CodSpeed claim.

Root reviewed the source and staged four immutable assets in a temporary DRAFT
release. The activated transport binds all four verified IDs and SHA-256 values.
Normal diagnostic publication is approved; root must review the exact published
commit before a single exact-SHA dispatch. Actual hosted results remain pending.
The release must never be published. Delete staging only after retained CI
evidence confirms consumption. No PAT or repository secret is used. Actual run 38950862496 proved that the
read-only Actions integration could not fetch the draft asset (HTTP 403), before
any controls or samples. A separate trusted download-only job now has
`contents: write`, performs GETs of only four hardcoded IDs, verifies bytes and
SHA-256, and uploads same-run artifacts. It never checks out or executes source
or native payloads. The dependent measurement VM remains read-only and checks
all four hashes again, plus repository/run/attempt/workflow-SHA ownership.
The staging release remains unpublished. No draft listing is used. Official endpoint permissions:
https://docs.github.com/en/rest/releases/assets#get-a-release-asset

Reuse the already registered `.github/workflows/receiver-fact-native.yml`
(workflow ID 384918609), on a separate diagnostic branch. Confirm no owner is
using that registration before dispatch. No new default-branch workflow PR is
needed. The replacement is dispatch-only and has its own concurrency group.

The 93 small payloads preserve the exact original frozen bytes at their original
absolute paths. Four pinned asset downloads complete the 97-file inventory.
The candidate input checkout is exact 099/tree d410; producer labels remain
c896 and 0076 plus its uncommitted d410 tree. Current e0 main is accepted only
through the already reviewed production/dependency/facade equivalence proof.

Before samples, both addons must pass all eight existing native tests and full
prepared-report equality. All eight CLI workload slots must match the original
full-byte goldens. Dynamic loads, Node version, image, CPU, memory, dependency
listing, live main/head/draft and completed exact-head Test CI are retained.
There is no local quiet lease or inference about external primary jobs.

The sampler differs only in its documentation and explicit isolated START
label. All 256 fixed ABBA/BAAB rows, deadlines, affinity, metrics, output checks,
and the assessor bytes are unchanged. The 600-second sample cap remains.
The adapter has a 1260-second command budget, and the job has a 30-minute cap;
downloads/controls phase fit is unproved until actual execution. All partial
logs and rows are uploaded even on failure. The Actions conclusion is not a
manual performance gate; root independently reproduces all 24 metric objects.
