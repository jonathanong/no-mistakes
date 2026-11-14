---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-github-actions]
append_system_prompt: |
  There is no repository in this environment. Do not read or search
  for files, and do not ask for any to be provided. Put your complete answer
  (the full code, or the full review) in your final message.
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
  snapshots:
    needs: test
    runs-on: ubuntu-latest
    timeout-minutes: 10
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: true # the push step below needs the token
      - run: corepack enable && pnpm install --frozen-lockfile
      - run: pnpm --filter api snapshots:update
      - run: git add -A && git diff --cached --quiet || (git commit -m "chore: update snapshots" && git push)
```
