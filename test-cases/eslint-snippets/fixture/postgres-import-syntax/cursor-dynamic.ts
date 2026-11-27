declare const flag: string;
declare const dyn: string;
const cur = require(flag);

cur.runCursor(dyn);

let { runCursor } = require(flag);
runCursor(dyn);
typeof runCursor;
export type { runCursor };
