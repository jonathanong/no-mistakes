import { beginTransaction, query } from "@example/db";
import { reserveJob } from "./helpers";

// lock-reason: prevents: double refund; writer: refund worker retry
const LOCK_REFUND_SQL = "SELECT id FROM refunds WHERE id = $1 FOR UPDATE";

export async function runAll(id: number) {
  // lock-reason: prevents: duplicate debit; writer: checkout retry handler
  await query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);

  await query(
    `
      -- lock-reason: prevents: overlapping shipment; writer: fulfillment worker
      SELECT id FROM shipments WHERE id = $1 FOR NO KEY UPDATE SKIP LOCKED
    `,
    [id],
  );

  await query(LOCK_REFUND_SQL, [id]);

  await beginTransaction(async (tx) => {
    /* lock-reason: prevents: concurrent payout; writer: payout cron */
    await tx.query("SELECT pg_advisory_xact_lock($1)", [id]);
  });

  // lock-reason: prevents: concurrent job reservation; writer: worker pool
  await reserveJob(id);

  await query("SELECT id FROM jobs WHERE id = $1", [id]);
}
