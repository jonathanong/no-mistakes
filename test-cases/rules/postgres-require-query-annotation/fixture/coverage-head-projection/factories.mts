import sql from "sql-template-strings";
import {add, replace} from "./effects.mjs";
import {add as reexportedAdd} from "./effect-reexport.mjs";
import {add as cyclicAdd} from "./effect-cycle-a.mjs";
import {add as absentAdd} from "./not-prepared.mjs";
import {add as brokenAdd} from "./broken.mjs";
export async function healthy() { const q = sql`/* healthy */ SELECT 1`; add(q); return q; }
export async function plain() { const q = sql`/* plain */ SELECT 1`; return q; }
export async function reexported() { const q = sql`/* reexported */ SELECT 1`; reexportedAdd(q); return q; }
export async function cyclic() { const q = sql`/* cyclic */ SELECT 1`; cyclicAdd(q); return q; }
export async function unresolved() { const q = sql`/* unresolved */ SELECT 1`; absentAdd(q); return q; }
export async function broken() { const q = sql`/* broken */ SELECT 1`; brokenAdd(q); return q; }
export async function replacement() { const q = sql`/* replacement */ SELECT 1`; replace(q); return q; }
export async function suspended() { const q = sql`/* suspended */ SELECT 1`; await pause; return q; }
export async function untrusted() { const q = unknownTag`/* untrusted */ SELECT 1`; return q; }
