---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-postgresql]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


runLedgerRollup(pool): we run six app instances and the rollup must
not run on two of them at once. use postgres for the guard, no new
infrastructure. node-postgres.

```ts
import type { Pool, PoolClient } from "pg";

// reads and writes ledger rows only, using the client it is given.
// takes a few seconds. returns how many rows it rolled up.
declare function rollup(client: PoolClient): Promise<number>;
```
