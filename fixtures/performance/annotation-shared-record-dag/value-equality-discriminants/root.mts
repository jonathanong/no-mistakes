import sql from 'sql-template-strings'
import { read } from '@bench/db'
// Equal SQL from distinct allocations must remain distinct builder candidates.
const left = sql`/* distinct identities */ SELECT 1 AS discriminant_identity_probe_with_a_deliberately_long_static_sql_prefix`
const right = sql`/* distinct identities */ SELECT 1 AS discriminant_identity_probe_with_a_deliberately_long_static_sql_prefix`
const selected = unknownCondition ? left : right
export function probe() {
  read(selected)
  read('SELECT 2') // Keep a concrete unsafe callsite alongside the annotation.
}
