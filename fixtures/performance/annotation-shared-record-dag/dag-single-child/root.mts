import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
// Shared record nodes model repeated references, not distinct child objects.
const q = sql`/* probe */ SELECT 1`
const a0 = { x: q }
const a1 = { x: a0 }
const a2 = { x: a1 }
const a3 = { x: a2 }
const a4 = { x: a3 }
const a5 = { x: a4 }
const a6 = { x: a5 }
const a7 = { x: a6 }
const a8 = { x: a7 }
const a9 = { x: a8 }
const a10 = { x: a9 }
const a11 = { x: a10 }
const a12 = { x: a11 }
const a13 = { x: a12 }
const a14 = { x: a13 }
const a15 = { x: a14 }
const a16 = { x: a15 }
const a17 = { x: a16 }
const a18 = { x: a17 }
const a19 = { x: a18 }
const a20 = { x: a19 }
const a21 = { x: a20 }
const a22 = { x: a21 }
const a23 = { x: a22 }
const a24 = { x: a23 }
const a25 = { x: a24 }
const a26 = { x: a25 }
const a27 = { x: a26 }
const a28 = { x: a27 }
const a29 = { x: a28 }
const a30 = { x: a29 }
const joined = unknownCondition ? a30 : a30
read(q)
