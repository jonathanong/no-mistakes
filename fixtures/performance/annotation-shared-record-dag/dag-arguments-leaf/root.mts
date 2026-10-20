import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
// The DAG holds argument identity only, without SQL builders.
function carry() { return arguments }
const leaf = carry(1)
const a0 = { x: leaf, y: leaf }
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
const a13 = { x: a12, y: a12 }
const a14 = { x: a13, y: a13 }
const a15 = { x: a14, y: a14 }
const a16 = { x: a15, y: a15 }
const a17 = { x: a16, y: a16 }
const a18 = { x: a17, y: a17 }
const a19 = { x: a18, y: a18 }
const a20 = { x: a19, y: a19 }
const a21 = { x: a20, y: a20 }
const a22 = { x: a21, y: a21 }
const a23 = { x: a22, y: a22 }
const a24 = { x: a23, y: a23 }
const a25 = { x: a24, y: a24 }
const a26 = { x: a25, y: a25 }
const a27 = { x: a26, y: a26 }
const a28 = { x: a27, y: a27 }
const a29 = { x: a28, y: a28 }
const a30 = { x: a29, y: a29 }
const joined = unknownCondition ? a30 : a30
read(sql`/* probe */ SELECT 1`)
