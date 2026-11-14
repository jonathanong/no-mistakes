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


review this before i merge. runs from cron every 10 minutes.

```ts
// src/holds/release-expired.ts
export async function releaseExpiredHolds(db: Db): Promise<number> {
  let released = 0;
  for (;;) {
    const { rowCount } = await db.query(
      `/* holds.release-expired */
       UPDATE holds SET released_at = now()
       WHERE id IN (
         SELECT id FROM holds
         WHERE released_at IS NULL AND expires_at < now()
         ORDER BY expires_at, id
         LIMIT 500
         FOR UPDATE SKIP LOCKED
       )`,
    );
    released += rowCount ?? 0;
    if (!rowCount) return released;
  }
}

export async function transferHold(
  db: Db,
  holdId: string,
  toAccountId: string,
): Promise<void> {
  await db.transaction(async (tx) => {
    // mutex: the holds_account_cap trigger counts an account's open holds on
    // UPDATE; without this, two concurrent transfers both see room under the cap.
    // The account row itself is neither read nor written.
    await tx.query(
      `/* holds.lock-account */
       SELECT id FROM accounts WHERE id = $1 FOR UPDATE`,
      [toAccountId],
    );
    await tx.query(
      `/* holds.transfer */
       UPDATE holds SET account_id = $2
       WHERE id = $1 AND released_at IS NULL`,
      [holdId, toAccountId],
    );
  });
}
```

`db.query` runs each call in its own transaction.
