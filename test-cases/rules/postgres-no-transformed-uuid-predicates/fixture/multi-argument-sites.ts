import { query } from 'db';
query('SELECT 1 FROM accounts WHERE fn(id, token) = $1 AND fn(id, token) = $2');
query(`SELECT 1 FROM accounts WHERE fn(
  id,
  token
) = $1`);
function makeSql() { return 'SELECT 1 FROM accounts WHERE COALESCE(id, token) = $1'; }
query(makeSql());
// no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates -- Whole call is intentionally transformed.
query('SELECT 1 FROM accounts WHERE fn(id, token) = $1');
