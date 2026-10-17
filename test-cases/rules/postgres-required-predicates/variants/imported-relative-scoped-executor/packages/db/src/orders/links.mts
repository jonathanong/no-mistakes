import sql from "sql-template-strings";
import { openTransaction } from "../transaction";
import type { TxExecutor } from "../transaction";
import { ownerVisibleSql } from "./visibility-sql.mts";

// Both executors are scoped through a relative import that resolves inside the
// configured package, and both consume SQL built by an imported helper.
export async function viaFactory(id: string) {
  const tx = await openTransaction();
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.owner_id"));
  return tx.query(query);
}

export async function viaParameter(run: TxExecutor, id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id"));
  return run.query(query);
}
