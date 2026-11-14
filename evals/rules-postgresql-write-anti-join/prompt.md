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


findWinbackCustomers(pool): customers who haven't ordered in the
last 90 days, for a win-back email. node-postgres. guest checkouts
leave orders.customer_id empty.

```sql
create table customers (
  id uuid primary key,
  email text not null,
  created_at timestamptz not null default now()
);
create table orders (
  id uuid primary key,
  customer_id uuid references customers(id),
  created_at timestamptz not null default now()
);
create index orders_customer_created_idx on orders (customer_id, created_at);
```
