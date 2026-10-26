import { query } from '@example/db'
// Recursive self references and the external binding identify one function.
function inner() { return 'unused outer' }
const makeSql = function inner(depth: number) {
  if (depth) return inner(depth - 1)
  return 'SELECT 1 LIMIT 2'
}
function wrap() { return makeSql(1) }
query(wrap())
