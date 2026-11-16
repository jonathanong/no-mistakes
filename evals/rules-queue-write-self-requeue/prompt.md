---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-queue]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


backfillSlugs(job): fill in the slug column for products that don't
have one yet, 500 at a time, and keep going until none are left.
one job shouldn't hold a worker for the whole backfill. node-postgres.

```ts
import type { Pool } from "pg";
import { backfillQueue } from "./queues"; // backfillQueue.add(name: string, data: object, opts?: { delay?: number; jobId?: string }): Promise<Job>

// products(id uuid, name text, slug text null)
// makeSlug(name: string): string  -- can return "" for names with no usable characters
declare const pool: Pool;
declare function makeSlug(name: string): string;
type BackfillJob = { data: { batch?: number } };
```
