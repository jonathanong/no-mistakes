import sql from 'sql-template-strings'
import { read } from '@bench/db'
async function names(label, other, a, b) {
  const first = sql`/* `.append(label).append(sql` */ SELECT 1`)
  const second = sql`/* `.append(other).append(sql` */ SELECT 2`)
  read(first) // proof: Literal
  // A cached first-parameter domain must never prove another parameter's root.
  read(second) // proof: Unproven
}
names('known', opaque, 1, 2)
