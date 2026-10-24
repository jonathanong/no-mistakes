import { read } from '@bench/db'
// Equal records have separate identities but duplicate their own descendants.
const left0 = "/* same */ SELECT 1"
const left1 = { a: left0, b: left0 }
const left2 = { a: left1, b: left1 }
const left3 = { a: left2, b: left2 }
const left4 = { a: left3, b: left3 }
const left5 = { a: left4, b: left4 }
const left6 = { a: left5, b: left5 }
const left7 = { a: left6, b: left6 }
const left8 = { a: left7, b: left7 }
const left9 = { a: left8, b: left8 }
const left10 = { a: left9, b: left9 }
const left11 = { a: left10, b: left10 }
const left12 = { a: left11, b: left11 }
const right0 = "/* same */ SELECT 1"
const right1 = { a: right0, b: right0 }
const right2 = { a: right1, b: right1 }
const right3 = { a: right2, b: right2 }
const right4 = { a: right3, b: right3 }
const right5 = { a: right4, b: right4 }
const right6 = { a: right5, b: right5 }
const right7 = { a: right6, b: right6 }
const right8 = { a: right7, b: right7 }
const right9 = { a: right8, b: right8 }
const right10 = { a: right9, b: right9 }
const right11 = { a: right10, b: right10 }
const right12 = { a: right11, b: right11 }
let chosen = left12
if (flag) chosen = right12
export function probe() {
  read("/* clean */ SELECT 1")
  return chosen
}
