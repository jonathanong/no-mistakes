import { query } from 'pg';
function execute() {
  const f = run;
  f('SELECT 1 OFFSET 1');
}
const run = (sql: string) => query(sql);
execute();
