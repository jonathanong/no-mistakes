<!-- cspell:ignore rlib cdylib codegen -->

# Hosted full-check performance

[Issue #1917](https://github.com/jonathanong/no-mistakes/issues/1917) concerns
the 180-second command deadline on GitHub-hosted `ubuntu-latest`. The separate
`Hosted full-check performance` workflow leaves the existing ARM CodSpeed and
local Criterion comparison contract unchanged.

The initial PR run builds source commit
`7c145f3e9fa800dbbeca5db0982c2b96d12b3e58`. After the workflow exists on the default
branch, dispatch it with `candidateRef` set to any exact 40-character source
commit. A new workflow cannot be dispatched before it exists on the default
branch, so its path-filtered pull request trigger supplies the first run.

Each of these public consumer commits gets six fresh `ubuntu-latest` jobs:

| Purpose | Commit | Git tree |
| --- | --- | --- |
| Acceptance | `22edf4bfe0a782ac0d030c916dedfd6f8acb499c` | `e848d7ad03e0c0fca7ec70eb0f678653f70c278b` |
| Reported passing/failing pair | `92d0a2061bde8d260aa0b86a2d71110ece1a43ca` | `11815b62e69a39be94ed790e8984994f50113cd3` |

The latter tree also belongs to merge-group commit
`c751f7ed7053844b344b9484fc7c100e6f51a74a`. It differs from the acceptance tree.
An older local Vouch checkout is not a substitute for either pinned tree.

Preparation follows the pinned consumer workflow: Node from `.nvmrc`, pnpm
`latest-12`, a complete frozen-lockfile install with lifecycle scripts, and its
pinned mise tools. Installation and the production build happen outside the
deadline. Each pair uses the unchanged consumer configuration and exact command:

```sh
pnpm exec no-mistakes --timeout 0 --lock-timeout 0 check
```

The published npm `0.86.2` launcher runs both variants. Only its untracked native
CLI is replaced for the candidate, then restored. The candidate build mirrors
the official x86 release flags, compiler pin, N-API build environment, and
temporary `rlib` plus `cdylib` manifest patch: fat LTO, one codegen unit, and all
default features including Mermaid. Build jobs are limited to one and swap is
added to protect the cold link; runtime jobs remain automatic. The baseline is
the published npm binary from the consumer lockfile, whose compiler is not
independently established. This comparison is published versus source-built,
not a claim that both binaries used identical compilers.

GNU timeout sends SIGTERM at 180 seconds to the command process group and allows
at most five seconds for termination before SIGKILL. A deadline exit or any
nonzero check exit remains a failed sample. Known baseline failures are retained
as measurements; candidate failures fail the pair and summary jobs. GNU time
records elapsed, user, system, maximum RSS in KiB, and exit. Elapsed time for a
stopped command includes termination cleanup; it is censored and must not be
presented as completed-check time. Raw stdout/stderr, time output, binary hashes,
source/compiler/build provenance, image version, CPU model, available CPUs, and
automatic Rayon job policy are uploaded even after candidate failure.

Resource maxima exclude timeout exits 124/137, signaled samples, and missing
wrapper statuses because descendant accounting may be incomplete. The resource
sample count includes completed nonzero exits; an empty resource sample reports
null JSON maxima and `n/a` in Markdown. Raw counters and deadline outcomes remain.

Each pair shares a runner and filesystem cache; binary order alternates across
jobs to reduce order bias. The report includes failed baseline measurements and
reports maximum and nearest-rank empirical p95 with the sample count. At six
pairs per tree p95 equals the maximum. These samples show observed behavior;
they cannot guarantee the hosted slow tail or establish a healthy runtime ceiling.
Phase logging on termination must be assessed from the archived candidate
stderr; this harness does not inject timing flags or change CLI output.
