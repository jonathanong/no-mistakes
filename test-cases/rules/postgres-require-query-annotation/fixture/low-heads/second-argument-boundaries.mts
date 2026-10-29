import sql from 'sql-template-strings'
import { read } from '@bench/db'

function safe(config, query) { query.append(config.tail) }
function tooMany(config, query, extra) { query.append(config.tail) }
function both(config, query) { config.text = 'SELECT 2'; query.append(' ') }
function boundaries(config) {
  // Appending can change EOF keyword/comment classification, so these are unresolved.
  const keyword = sql`BEGIN`
  safe({ tail: 'NING' }, keyword)
  read(keyword) // low: false
  const comment = sql`/* partial`
  safe({ tail: ' */ SELECT 1' }, comment)
  read(comment) // low: false
  const missing = sql`BEGINNING`
  safe(config, missing)
  read(missing) // low: false
  const threeFormals = sql`/* formals */ SELECT ${config.value}`
  tooMany(config, threeFormals, true)
  read(threeFormals) // low: false
  const threeActuals = sql`/* actuals */ SELECT ${config.value}`
  safe(config, threeActuals, true)
  read(threeActuals) // low: false
  const duplicate = sql`/* duplicate */ SELECT ${config.value}`
  both(duplicate, duplicate)
  read(duplicate) // low: false
}
