import { query } from '@example/db'
function label() { return 'unused outer' }
function wrapper() { return label() }
function run() {
  function label() { return 'SELECT 1 LIMIT 2' }
  function wrapper() { return label() }
  query(wrapper())
}
