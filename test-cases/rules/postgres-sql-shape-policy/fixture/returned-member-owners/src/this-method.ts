import { query } from '@example/db'
class Repo {
  q() {
    return 'SELECT 3 LIMIT 4'
  }
  run() { return query(this.q()) }
}
// Lexically keyed unused helpers must still be filtered.
function unusedSql() { return 'SELECT 9 LIMIT 10' }
const unusedBound = function inner() { return 'SELECT 11 LIMIT 12' }
