import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
// Shared record nodes model repeated references, not distinct child objects.
const q = sql`/* probe */ SELECT 1`
const a0 = { x: q, y: q }
const a1 = { x: a0, y: a0 }
const a2 = { x: a1, y: a1 }
const a3 = { x: a2, y: a2 }
const a4 = { x: a3, y: a3 }
const a5 = { x: a4, y: a4 }
const a6 = { x: a5, y: a5 }
const a7 = { x: a6, y: a6 }
const a8 = { x: a7, y: a7 }
const a9 = { x: a8, y: a8 }
const a10 = { x: a9, y: a9 }
const a11 = { x: a10, y: a10 }
const a12 = { x: a11, y: a11 }
const joined = unknownCondition ? a12 : a12
read(q)
