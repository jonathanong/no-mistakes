import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
// Aliases stay reference-bearing so binding pruning cannot erase the batching workload.
const query = sql`/* batched */ SELECT 1`
const alias0 = query
const alias1 = query
const alias2 = query
const alias3 = query
const alias4 = query
const alias5 = query
const alias6 = query
const alias7 = query
const alias8 = query
const alias9 = query
const alias10 = query
const alias11 = query
const alias12 = query
const alias13 = query
const alias14 = query
const alias15 = query
const alias16 = query
const alias17 = query
const alias18 = query
const alias19 = query
const alias20 = query
const alias21 = query
const alias22 = query
const alias23 = query
const alias24 = query
const alias25 = query
const alias26 = query
const alias27 = query
const alias28 = query
const alias29 = query
const alias30 = query
const alias31 = query
const joined = unknownCondition ? query : query
read(query)
read(sql`SELECT 2`) // unsafe-query
