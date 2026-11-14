---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-queue]
append_system_prompt: |
  There is no repository in this environment. Do not read or search
  for files, and do not ask for any to be provided. Put your complete answer
  (the full code, or the full review) in your final message.
---


review this before i merge

```ts
// src/webhooks/receive.ts
export async function receiveWebhook(event: { id: string; type: string }) {
  await db.transaction(async (tx) => {
    // The receipt row is the only record of this delivery. Enqueue stays inside the
    // transaction so a failed enqueue rolls the row back and the provider retries.
    // jobId = event id, so a duplicate delivery collides harmlessly.
    await tx
      .insert(webhookReceipts)
      .values({ eventId: event.id, type: event.type })
      .onConflictDoNothing();
    await webhookQueue.add("process-webhook", { eventId: event.id }, { jobId: event.id });
  });
}

// src/webhooks/worker.ts
export async function processWebhook(job: { data: { eventId: string } }) {
  const receipt = await findReceipt(job.data.eventId);
  if (!receipt) throw new Error("receipt not visible yet; retry");
  if (receipt.processedAt) return;
  await applyEvent(receipt);
  await markProcessed(receipt.eventId);
}
```
