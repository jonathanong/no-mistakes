import { query } from 'db';
function getColumn() { return 'id'; }
function prefixSql() {
  return 'SELECT 1 FROM accounts WHERE CAST(' + getColumn() + ' AS text) = $1 AND CAST(' + getColumn() + ' AS text) = $2';
}
function postfixSql() {
  return 'SELECT 1 FROM accounts WHERE ' + getColumn() + '::text = $1 AND ' + getColumn() + '::text = $2';
}
query(prefixSql());
query(postfixSql());
