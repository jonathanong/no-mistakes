import { query } from 'db';

function buildSql(column: string) {
  return 'SELECT 1 FROM accounts WHERE fn(id, ' + column + ') = $1';
}
query(buildSql('email'));
query(buildSql('token'));
