import { query } from '@example/db'
// This unused outer string must not be reached by the inner SQL call.
function label() { return '.mjs' }
function run() {
  function label() { return 'SELECT 1 LIMIT 2' }
  query(label())
}
