import sql from 'sql-template-strings'
import { read } from '@bench/db'
async function positive(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Literal
  read(query) // proof: Literal
  read(query) // proof: Literal
}
positive('one', 1, 2, 3)
positive('two', 1, 2, 3)
positive('three', 1, 2, 3)
async function negative(label, a, b, c) {
  // Two roots share the same parameter owner; both fail its complete-use census.
  const first = sql`/* `.append(label).append(sql` */ SELECT 1`)
  const second = sql`/* `.append(label).append(sql` */ SELECT 2`)
  read(first) // proof: Unproven
  read(second) // proof: Unproven
}
negative('one', 1, 2, 3)
negative('two', 1, 2, 3)
