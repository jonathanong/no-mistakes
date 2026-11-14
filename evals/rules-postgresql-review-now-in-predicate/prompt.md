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


review this before i merge. runs from cron every minute.

```ts
// src/holds/repository.ts
export async function releaseExpiredHolds(db: Db): Promise<number> {
  const { rowCount } = await db.query(
    `/* holds.release-expired */
     UPDATE holds SET released_at = now()
     WHERE id IN (
       SELECT id FROM holds
       WHERE released_at IS NULL AND expires_at < now()
       ORDER BY expires_at
       LIMIT 500
       FOR UPDATE SKIP LOCKED
     )`,
  );
  return rowCount ?? 0;
}

export async function transferHold(
  db: Db,
  holdId: string,
  toAccountId: string,
): Promise<void> {
  await db.transaction(async (tx) => {
    // mutex: serializes transfers into one account; this row is neither read nor written
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
