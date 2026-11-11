# no-mistakes-rules

Short lists of things a coding agent must not do. Each skill covers one topic
and loads only when that topic comes up.

- **MUST NOT**: a defect. Fix it before handoff.
- **SHOULD NOT**: allowed only with a stated, measured reason.

| Skill | Covers |
|---|---|
| `testing-rules` | Assertions, mocks, isolation, timing, fixtures, snapshots |
| `playwright-rules` | Browser tests: waits, locators, test ids, setup |
| `postgresql-rules` | Queries, locks, transactions, schema, partitioning |
| `queue-rules` | Producers, workers, leases, sweeps, bounded iteration |
| `typescript-rules` | Async, errors, mutation, dead paths, Node IO, Redis, packages |
| `react-rules` | React and Next.js |
| `rust-rules` | Lints, tests, parallelism, hot loops, builds |
| `github-actions-rules` | Pinning, permissions, timeouts, polling, required checks |
| `dependency-rules` | Manifests, lockfiles, publishing, Dependabot |
| `security-rules` | Secrets, auth, SSRF, injection, API shape |
| `git-rules` | Commits, rebases, PRs, docs |
| `infra-rules` | Infrastructure as code, ops scripts, shell |
| `native-app-rules` | Swift and .NET apps |

Many rules are enforced by the `no-mistakes` CLI checks and
`eslint-plugin-no-mistakes`; this plugin states them for code an agent is about
to write, before any check runs. Install the `no-mistakes` plugin for the
analysis commands.
