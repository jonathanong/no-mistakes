import { query } from '@example/db'
class Repo {
  static q() {
    return 'SELECT 3 LIMIT 4'
  }
}
query(Repo.q())
// Lexically keyed unused helpers must still be filtered.
function unusedSql() { return 'SELECT 9 LIMIT 10' }
const unusedBound = function inner() { return 'SELECT 11 LIMIT 12' }
