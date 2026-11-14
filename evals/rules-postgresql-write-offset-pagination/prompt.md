---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-postgresql]
append_system_prompt: |
  There is no repository in this environment. Do not read or search
  for files, and do not ask for any to be provided. Put your complete answer
  (the full code, or the full review) in your final message.
---


need listOrders for the orders screen. node-postgres, newest first,
25 per page, the UI has next/prev buttons. pool is exported from ./db.

```sql
create table orders (
  id uuid primary key default uuidv7(),
  customer_id uuid not null references customers(id),
  total_cents int not null,
  created_at timestamptz not null default now()
);
create index orders_customer_created_idx
  on orders (customer_id, created_at desc, id desc);
```
