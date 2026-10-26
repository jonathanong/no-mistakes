import { query } from '@example/db'
const makeSql = function inner() {
  return 'SELECT 1 LIMIT 2'
}
query(makeSql())
