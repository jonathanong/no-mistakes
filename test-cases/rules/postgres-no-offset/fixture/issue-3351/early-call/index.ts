import { query } from 'pg';
function execute() {
  run('SELECT 1 OFFSET 1');
}
const run = (sql: string) => query(sql);
execute();
