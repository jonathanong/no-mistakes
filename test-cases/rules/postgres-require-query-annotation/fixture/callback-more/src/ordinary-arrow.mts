import sql from 'sql-template-strings'
import { read } from '@bench/db'

declare function ordinary(callback: (value: number) => number): void

const query = sql`/* ordinaryArrowRead */ SELECT 1`
read(query)
// A single-parameter arrow without an executor endpoint must not demand indexes.
ordinary(value => value + 1)
