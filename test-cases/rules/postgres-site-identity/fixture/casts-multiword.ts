import { query } from 'db';
function getColumn() { return 'id'; }
function prefixSql() {
  return 'SELECT 1 FROM accounts WHERE CAST(' + getColumn() + ' AS character varying(36)) = $1 AND CAST(' + getColumn() + ' AS character varying(36)) = $2';
}
function postfixSql() {
  return 'SELECT 1 FROM accounts WHERE ' + getColumn() + '::character varying(36) = $1 AND ' + getColumn() + '::character varying(36) = $2';
}
query(prefixSql());
query(postfixSql());
