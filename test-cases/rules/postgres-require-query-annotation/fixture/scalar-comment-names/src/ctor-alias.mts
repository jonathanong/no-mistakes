import sql from 'sql-template-strings'
import { read } from '@bench/db'
const alias = sql; consume(alias)
async function rejected(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
rejected('known', 1, 2, 3)
