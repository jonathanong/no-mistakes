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


review this before i merge

```yaml
name: build
on:
  pull_request:
    branches: [main]
permissions:
  contents: read
jobs:
  wasm:
    # 8-core runner: this build is 14 min on the standard runner and 4 min here,
    # measured on the last ten main builds.
    runs-on: ubuntu-24.04-8-core
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - run: make wasm
  ios:
    # macOS is required: xcodebuild does not run on Linux.
    runs-on: macos-15
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          persist-credentials: false
      - run: xcodebuild -scheme App -destination 'platform=iOS Simulator,name=iPhone 16' test
```
