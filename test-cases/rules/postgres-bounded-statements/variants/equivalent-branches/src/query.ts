import { query, sql } from "@example/db";
const shared = sql`/* same */`;
// Five independent redundant choices still produce one complete bounded query.
query(sql`SELECT id FROM accounts ${a ? shared : shared} ${b ? shared : shared} ${c ? shared : shared} ${d ? shared : shared} ${e ? shared : shared} LIMIT 1`);
const first = gateA ? true : false;
const second = gateB ? true : false;
const third = gateC ? true : false;
const fourth = gateD ? true : false;
const fifth = gateE ? true : false;
// A recovered boolean guard contributes no SQL when both arms share a value.
query(sql`SELECT id FROM accounts ${first ? shared : shared} ${second ? shared : shared} ${third ? shared : shared} ${fourth ? shared : shared} ${fifth ? shared : shared} LIMIT 1`);
