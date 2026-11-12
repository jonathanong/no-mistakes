import { read } from '@bench/db'
import sql from 'sql-template-strings'

// The captured loop binding is one primitive literal per iteration.
for (const [scenario, definition] of [
  ['freshness', 'SELECT 1'],
  ['retention', 'SELECT 2'],
] as const) {
  schedule(async () => {
    read(`/* ${scenario} */ ${definition}`) // low: true
  })
}

async function run() {
  const select = async (branch: string) => findRows(branch)
  await Promise.all([
    select('updated_at'),
    select('effective_at'),
    select('expires_at'),
  ])
}
async function findRows(branch: string) {
  const query = sql``.append(`/* fixed:${branch} */ SELECT 1`)
  query.append(runtimeTail)
  read(query) // low: true
}

// Ordinary templates accept a dynamic name; the tagged builder still needs a finite source.
function unknown(branch: string) {
  read(`/* fixed:${branch} */ SELECT 1`) // low: true
  read(sql`/* fixed:${branch} */ SELECT 1`) // low: false
}
