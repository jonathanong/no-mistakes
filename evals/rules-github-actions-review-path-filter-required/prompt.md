---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-github-actions]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


review this before i merge. it's the new backend CI; "backend / test"
is a required check on main.

```yaml
name: backend
on:
  pull_request:
    branches: [main]
    paths:
      - "apps/api/**"
      - "packages/db/**"
permissions:
  contents: read
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4.4.0
        with:
          node-version: 24
      - run: corepack enable && pnpm install --frozen-lockfile
      - run: pnpm --filter api test
  integration:
    needs: test
    # Measured: 31 min on ubuntu-latest, 9 min here (parallel Postgres shards).
    runs-on: ubuntu-latest-8-cores
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4.4.0
        with:
          node-version: 24
      - run: corepack enable && pnpm install --frozen-lockfile
      - run: pnpm --filter api test:integration
```
