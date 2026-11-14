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


createInvoice(input): insert the invoice row and its line items,
then queue a send-invoice job with the invoice id so the email goes
out. helpers below.

```ts
import { db } from "./db"; // db.transaction(async (tx) => ...) commits when the callback resolves, rolls back if it throws
import { invoiceQueue } from "./queues"; // invoiceQueue.add(name: string, data: object, opts?: { jobId?: string }): Promise<Job>
import { invoices, invoiceLines } from "./schema"; // tx.insert(table).values(row | rows).returning()

type Input = {
  customerId: string;
  lines: Array<{ sku: string; qty: number; unitCents: number }>;
};
```
