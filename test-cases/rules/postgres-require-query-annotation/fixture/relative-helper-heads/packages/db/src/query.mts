import sql from 'sql-template-strings';
import { read } from '@example/db';
import { openTransaction, type TxExecutor } from './transaction.mjs';
import { openTransaction as otherOpen } from '../../other/src/transaction.mjs';

function fixed() { return sql`/* relativeNamed */ SELECT 1`; }
function missing() { return sql`SELECT 1`; }

function run() {
  const tx = openTransaction();
  tx.query(fixed()); // accepted
  tx.query(missing()); // missing
  const typed: TxExecutor = tx;
  typed.query(fixed()); // accepted

  // This relative import resolves outside @example/db and cannot excuse q's escape.
  const other = otherOpen();
  const q = sql`/* local */ SELECT 1`;
  other.query(q);
  read(q); // unanalyzable
}
