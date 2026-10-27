import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
// Keep unrelated scalars and functions: the immutable binding index skips them.
const scalar0 = 0
const scalar1 = 1
const scalar2 = 2
const scalar3 = 3
const scalar4 = 4
const scalar5 = 5
const scalar6 = 6
const scalar7 = 7
const scalar8 = 8
const scalar9 = 9
const scalar10 = 10
const scalar11 = 11
const scalar12 = 12
const scalar13 = 13
const scalar14 = 14
const scalar15 = 15
const scalar16 = 16
const scalar17 = 17
const scalar18 = 18
const scalar19 = 19
const scalar20 = 20
const scalar21 = 21
const scalar22 = 22
const scalar23 = 23
const scalar24 = 24
const scalar25 = 25
const scalar26 = 26
const scalar27 = 27
const scalar28 = 28
const scalar29 = 29
const scalar30 = 30
const scalar31 = 31
const callback = () => sql`SELECT 3`
const query = sql`/* indexed */ SELECT 1`
const holder = { query }
const joined = unknownCondition ? (0, holder) : (0, holder) // Sequence arms deliberately exercise the full join/traversal path.
const selected = unknownCondition ? (0, callback) : (0, callback)
read(joined.query)
read(callback()) // unsafe-query
read(sql`SELECT 2`) // unsafe-query
