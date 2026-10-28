const sql = require('sql-template-strings')
const { read } = require('@bench/db')
let { append } = require('./effects.mjs')
append = opaque
function changed(items, tail) {
  const q = sql`/* changedRequire */ SELECT 1`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q)
}
