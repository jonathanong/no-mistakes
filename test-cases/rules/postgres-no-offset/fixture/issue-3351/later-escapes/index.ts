import { query } from 'pg';
const callback = (sql: string) => query(sql);
function invoke(run: (sql: string) => unknown) {
  run('SELECT 1 OFFSET 1');
}
invoke(callback);

const aliasTarget = (sql: string) => query(sql);
const alias = aliasTarget;
alias('SELECT 2 OFFSET 1');

const memberTarget = (sql: string) => query(sql);
memberTarget.call(null, 'SELECT 3 OFFSET 1');

export const exported = (sql: string) => query(sql);
