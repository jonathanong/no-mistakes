import { query } from "@example/db";
import { reserveJob } from "./helpers";

// lock-reason: prevents: duplicate ledger entry; writer: ledger worker
const LEDGER_SQL = `SELECT acquire_ledger_lock($1)`;

export async function runLocalOrigin(id: number) {
  await query(LEDGER_SQL, [id]);
}

export async function runRegexExtent(id: number) {
  reserveJob(
    /[)]/,
    /* lock-reason: prevents: duplicate reservation; writer: worker pool */
    id,
  );
}
