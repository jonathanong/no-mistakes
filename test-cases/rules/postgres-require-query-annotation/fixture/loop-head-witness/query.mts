import { read } from "@bench/db";
import sql from "sql-template-strings";
import * as namespace from "sql-template-strings";

function valid(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = "SELECT 2"; } }
function validTemplate(xs) { let q = `/* named */ SELECT 1`; for (const x of xs) { read(q); q = `SELECT 2`; } }
function validTag(xs) { let q = sql`/* named */ SELECT 1`; for (const x of xs) { read(q); q = sql`SELECT 2`; } }
function validNamespace(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = namespace`SELECT 2`; } }
// Unsupported forms deliberately retain their old outcome instead of guessing a backedge.
async function awaited(xs) { let q = "/* named */ SELECT 1"; for await (const x of xs) { read(q); q = "SELECT 2"; } }
function iterableCall(xs) { let q = "/* named */ SELECT 1"; for (const x of xs()) { read(q); q = "SELECT 2"; } }
function assignmentLeft(xs, x) { let q = "/* named */ SELECT 1"; for (x of xs) { read(q); q = "SELECT 2"; } }
function destructuring(xs) { let q = "/* named */ SELECT 1"; for (const [x] of xs) { read(q); q = "SELECT 2"; } }
function noBlock(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) read(q); }
function tooLong(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = "SELECT 2"; break; } }
function wrongFirst(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { q; q = "SELECT 2"; } }
function optional(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read?.(q); q = "SELECT 2"; } }
function propertyExecutor(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { db.read(q); q = "SELECT 2"; } }
function twoArgs(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q, 1); q = "SELECT 2"; } }
function spreadArg(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(...q); q = "SELECT 2"; } }
function literalArg(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read("/* named */ SELECT 1"); q = "SELECT 2"; } }
function wrongSecond(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q; } }
function compound(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q += "SELECT 2"; } }
function propertyAssignment(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); object.q = "SELECT 2"; } }
function interpolated(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = `SELECT ${x}`; } }
function interpolatedTag(xs) { let q = sql`/* named */ SELECT 1`; for (const x of xs) { read(q); q = sql`SELECT ${x}`; } }
function propertyTag(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = object.sql`SELECT 2`; } }
function notSql(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = 42; } }
function annotated(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = "/* named */ SELECT 2"; } }
function transaction(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = "COMMIT"; } }
function mismatchedBinding(xs) { let q = "/* named */ SELECT 1", other; for (const x of xs) { read(q); other = "SELECT 2"; } }
function loopBinding(xs) { for (let q of xs) { read(q); q = "SELECT 2"; } }
function localTag(xs, local) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = local`SELECT 2`; } }
let top = "/* named */ SELECT 1";
for (const x of xs) { read(top); top = "SELECT 2"; }
function escaped(xs) { let q = sql`/* named */ SELECT 1`; stash(q); for (const x of xs) { read(q); q = sql`SELECT 2`; } }
function unrelatedExecutor(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { consume(q); q = "SELECT 2"; } }
function constant(xs) { const q = "SELECT 1"; for (const x of xs) { read(q); q = "SELECT 2"; } }
function unboundRead(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(unbound); q = "SELECT 2"; } }
function unboundWrite(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); unbound = "SELECT 2"; } }
function unboundTag(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = unbound`SELECT 2`; } }

async function validAwait(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { await read(q); q = "SELECT 2"; } }
