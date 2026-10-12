## Hosted full-check measurements

- Keep the existing ARM CodSpeed/Criterion contract. The separate hosted check
  workflow measures issue #1917 on `ubuntu-latest`, using both the acceptance
  commit and the distinct passing/failing tree pinned in that issue.
- Preserve the consumer's configuration, default features (including Mermaid),
  and exact public check command. Install/build/tool preparation is outside the
  180-second command deadline. A timeout or nonzero check exit remains a failure.
- Record source/binary provenance, compiler/flags, runner image/CPU, available
  CPUs, automatic Rayon jobs, elapsed/user/system time, RSS, exit, and raw logs.
  Report sample count with max/p95; a small hosted sample cannot establish a
  guaranteed tail bound. Disclose wrapper/compiler/build differences and cache
  warming/order effects. Never substitute an older local consumer fixture.
- Timed-out samples are censored; do not infer completed-runtime percentages.
  Killed-wrapper CPU/RSS can omit descendants it has not waited for; exclude those
  values from completed-run resource comparisons. For six samples, nearest-rank
  empirical p95 is the observed maximum, subject to the tail-bound limitation above.
