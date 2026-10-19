# #2238 ordering-only ARM measurement

Pushing `eval/2238-current-arm` starts `current-verdict-ordering-arm.yml`. The
workflow pins main base `c009d087f3eb32b27a27d6a200154e26b7be0a5d` and candidate
`1f65a505367720f4519e486d300036fad4eb3adc`. It builds the selected frozen
main base and candidate sequentially on one
`ubuntu-24.04-arm` VM with the same stable Rust, default features, locked
workspace, release profile, and two Cargo build jobs. It retains fresh binaries,
CPU probes, and source/build provenance; it never reuses historical CLI builds.

Twelve independent matrix blocks cover jobs 1, 4, and default, diagnostics off
and on, and the candidate SQL verdict ledger off and on. Each block runs B-C-C-B
on one VM. The baseline always uses the ledger flag off. Each 3600-second arm
has the same installed consumer, derived ordering-only config, and physical
source tree. The config is written under the runner temp directory, and the
fixture's original config and Git-clean workload stay unchanged.

Parity requires identical repeated raw report hashes, exit codes, and semantic
stderr. With the ledger off, only the two saved `postgres-conflict-ordering`
findings may be added. With the ledger on, the ledger is schema-validated and
removed from only the candidate report before applying the same exact two-finding
delta check. Any additional field, warning, or finding difference fails closed
and leaves raw reports and stderr available as artifacts. A censored or
incomplete BCCB block also fails closed and retains every completed arm.

The pins identify the frozen local candidate and base. Do not launch until root
reviews the completed source coverage and explicitly authorizes the workflow.

This is a focused ordering-only configuration, not a full-config or
full-consumer gate measurement. It cannot clear a full-config jobs-1 gate. All
ratios are observations for this selected rule scope, not gate evidence.

Local validation:

```sh
python3 -B -m unittest discover -s .github/scripts/matched-arm -p 'test_*.py'
python3 -B -m unittest discover -s .github/scripts/current-verdict-arm -p 'test_*.py'
actionlint .github/workflows/current-verdict-ordering-arm.yml
```

## Measure-only recheck

The separate `current-verdict-ordering-recheck-arm.yml` triggers only on
`eval/2238-ordering-recheck-arm-main`. It never builds or changes the original
`eval/2238-current-arm` workflow. It reuses only the fresh production CLI pair
from builder run `39293039433`, builder harness commit
`26c28f246c03ce7f235b7d1d8765e96a11005aed`, and the same frozen source pins above.
The new measurement harness SHA is recorded separately; it must not be substituted
for the original builder SHA inside downloaded build proofs.

The launch manifest `fixtures/performance/current-verdict-arm/recheck-artifact.json`
pins the root-verified artifact `12209251738`, outer ZIP SHA256
`c89044cc9489bf84b51d7bdc041dea8500908da51eeeab7963df96f2003adfb4`, all twelve
file hashes, both independently Git-verified source trees, and matched build
fields. The saved metadata records expiry `2026-10-26T17:18:52Z`; runtime checks
still require a fresh unexpired API snapshot and successful original build job.
The incomplete `recheck-artifact-pending.json` remains a negative test fixture;
synthetic fixtures under `recheck-contract/` are never launch metadata.
The original matrix need not finish. No readiness polling or recursive dispatch exists.

Each recheck independently verifies artifact origin/expiry, actual downloaded
ZIP digest, safe ZIP paths, exact CLI file inventory/hashes, and source/build
proofs before any executable runs. Consumer outer ZIP SHA256 and inner
`fixture.tar.gz` SHA256 are different identities and are both retained. The
restored consumer must still match its physical provenance and original config.
The same twelve ordering-only BCCB blocks, early report parity, 3600-second arm
cap, and `eligibleForGate: false` remain in force. The 270-minute job covers
261 minutes of declared steps. User-authorized uninterrupted consumer arms take
precedence over the generic 30-minute Actions skill default; there are no hidden
background operations. This remains selected-rule evidence, never a full gate.
