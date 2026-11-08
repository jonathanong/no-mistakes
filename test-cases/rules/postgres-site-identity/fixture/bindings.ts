import { query } from 'db';
function buildSql(column: string) {
  return 'SELECT ' + column + ' FROM accounts WHERE fn(id, ' + column + ') = $1';
}
const first = buildSql('id');
const second = buildSql('token');
query(first);
query(second);
