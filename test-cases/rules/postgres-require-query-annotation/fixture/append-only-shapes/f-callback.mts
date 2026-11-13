import sql from 'sql-template-strings'
import { read } from '@bench/db'
function addPage(q, id) { q.append(sql` AND id = ${id}`) }
export function callback(id) {
  pageRunner(() => {
    const q = sql`/* callback */ SELECT 1 WHERE true`
    addPage(q, id)
    read(q)
  })
}
