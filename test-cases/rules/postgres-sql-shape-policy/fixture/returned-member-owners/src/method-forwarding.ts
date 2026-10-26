import { query } from '@example/db'
// Unknown member owners conservatively retain their returned helper calls.
function makeSql() {
  return 'SELECT 3 LIMIT 4'
}
class Repo {
  q() { return makeSql() }
  run() {
    // Anonymous callbacks nested in a member do not inherit its fallback.
    [1].map(() => { return '-' })
    [1].map(() => { return unusedSql() })
    return query(this.q())
  }
}
// Unrelated lexical helpers must still be filtered.
function unusedSql() { return 'SELECT 9 LIMIT 10' }
const unusedBound = function inner() { return 'SELECT 11 LIMIT 12' }
