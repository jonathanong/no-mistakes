import { query } from "@example/db";
import { reserveJob } from "./helpers";

export async function unreasoned(id: number) {
  await reserveJob(id);
  await query("SELECT id FROM refunds WHERE id = $1 FOR UPDATE", [id]);
  await query("SELECT pg_advisory_xact_lock($1)", [id]);

  // lock-reason: prevents: double charge
  await query("SELECT id FROM invoices WHERE id = $1 FOR SHARE", [id]);

  // no-mistakes: deadlock-safe -- ordered by account_id
  await query("SELECT id FROM accounts WHERE id = $1 FOR UPDATE", [id]);
}
