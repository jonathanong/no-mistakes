Matched ARM run: https://github.com/jonathanong/no-mistakes/actions/runs/39488578982

Main `4d4deb9423111f6bd97f2501bdf508ef17536dc0`; candidate `bcbbf58d69f8157aab60e5f1e61decf68ad278b5`.

Three same-VM main/main A/A blocks followed by three balanced baseline/candidate/candidate/baseline blocks in every cell. Six adjacent pairs per stage; same default features, fat LTO, one code generation unit and two build jobs. Ordering-only cohort keeps annotation evaluation off the critical path and preserves the non-rule payload and config-parent .gitignore.

RSS is the mean of per-process peak RSS. Deltas are mean adjacent paired percentages; A/A is maximum observed absolute paired wall spread, a finite descriptive sample rather than a guaranteed noise bound.

Ledger off:

| Jobs    | Diagnostics | Wall seconds main → candidate (Δ) | CPU seconds main → candidate (Δ) | Peak RSS MiB main → candidate (Δ) | A/A wall spread |
| ------- | ----------- | --------------------------------- | -------------------------------- | --------------------------------- | --------------- |
| 1       | off         | 10.408 → 10.527 (+1.15%)          | 10.411 → 10.529 (+1.15%)         | 1444.758 → 1445.852 (+0.08%)      | ±1.67%          |
| 1       | on          | 10.514 → 10.514 (+0.01%)          | 10.518 → 10.519 (+0.01%)         | 1450.635 → 1451.913 (+0.09%)      | ±0.57%          |
| 4       | off         | 5.560 → 5.567 (+0.16%)            | 10.990 → 11.040 (+0.48%)         | 1637.206 → 1638.418 (+0.07%)      | ±1.29%          |
| 4       | on          | 5.546 → 5.565 (+0.34%)            | 11.018 → 11.055 (+0.34%)         | 1642.171 → 1644.717 (+0.16%)      | ±3.04%          |
| max (4) | off         | 5.626 → 5.672 (+0.84%)            | 11.245 → 11.303 (+0.54%)         | 1636.488 → 1638.605 (+0.13%)      | ±7.61%          |
| max (4) | on          | 5.596 → 5.637 (+0.73%)            | 11.107 → 11.149 (+0.38%)         | 1643.233 → 1644.044 (+0.05%)      | ±1.70%          |

Ledger on:

| Jobs    | Diagnostics | Wall seconds main → candidate (Δ) | CPU seconds main → candidate (Δ) | Peak RSS MiB main → candidate (Δ) | A/A wall spread |
| ------- | ----------- | --------------------------------- | -------------------------------- | --------------------------------- | --------------- |
| 1       | off         | 10.244 → 10.352 (+1.05%)          | 10.245 → 10.353 (+1.05%)         | 1444.548 → 1446.221 (+0.12%)      | ±1.23%          |
| 1       | on          | 10.561 → 10.682 (+1.15%)          | 10.565 → 10.686 (+1.15%)         | 1450.507 → 1451.921 (+0.10%)      | ±1.58%          |
| 4       | off         | 5.254 → 5.334 (+1.53%)            | 10.372 → 10.505 (+1.28%)         | 1636.596 → 1647.893 (+0.69%)      | ±2.03%          |
| 4       | on          | 5.629 → 5.655 (+0.46%)            | 11.095 → 11.191 (+0.90%)         | 1643.306 → 1652.641 (+0.57%)      | ±3.32%          |
| max (4) | off         | 5.502 → 5.625 (+2.24%)            | 11.031 → 11.186 (+1.42%)         | 1637.644 → 1647.915 (+0.63%)      | ±1.86%          |
| max (4) | on          | 5.658 → 5.819 (+2.85%)            | 11.244 → 11.472 (+2.03%)         | 1643.391 → 1653.394 (+0.61%)      | ±2.84%          |

All 288 raw arms finished uncensored with matching output/exit proofs. The two intended feature findings and all 2,155 ledger records are independently checked. No full-config or newer-main performance claim.
