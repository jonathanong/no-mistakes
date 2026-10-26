import { query } from '@example/db'
const helpers = {
  h() {
    return 'SELECT 4 LIMIT 5'
  }
}
query(helpers.h())
// Lexically keyed unused helpers must still be filtered.
function unusedSql() { return 'SELECT 9 LIMIT 10' }
const unusedBound = function inner() { return 'SELECT 11 LIMIT 12' }
