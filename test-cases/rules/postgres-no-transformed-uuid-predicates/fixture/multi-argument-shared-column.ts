import { query } from 'db';

function getColumn() { return 'id'; }
function buildSql() {
  return 'SELECT 1 FROM accounts WHERE fn(' + getColumn() + ') = $1 AND fn(' + getColumn() + ') = $2';
}
query(buildSql());
