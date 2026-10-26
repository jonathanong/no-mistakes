import { query } from '@example/db'
async function makeSql() {
  return 'SELECT 1 LIMIT 2'
}
query(await makeSql())
