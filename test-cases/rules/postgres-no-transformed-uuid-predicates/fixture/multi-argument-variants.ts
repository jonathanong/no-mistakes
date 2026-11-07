import { query } from 'db';

function buildSql(flag: boolean) {
  return 'SELECT 1 FROM accounts WHERE fn(id, ' + (flag ? 'token' : 'email') + ') = $1';
}

export function run(flag: boolean) {
  return query(buildSql(flag));
}
