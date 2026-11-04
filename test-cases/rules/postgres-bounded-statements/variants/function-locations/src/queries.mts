import sql from 'sql-template-strings'
import { query } from '@example/db'
import { relation } from './relation.mts'

declare const limit: number

await query(sql`
SELECT unknown_fn(id)
FROM accounts
LIMIT ${limit}
`)

await query(sql`
SELECT set_fn(id)
FROM accounts
LIMIT ${limit}
`)

await query(sql`
SELECT unknown_fn(id)
FROM accounts LIMIT ${limit}`) // no-mistakes-disable-line postgres-bounded-statements

await query(sql`
SELECT set_fn(id)
FROM accounts LIMIT ${limit}`) // no-mistakes-disable-line postgres-bounded-statements

await query(relation(limit))
