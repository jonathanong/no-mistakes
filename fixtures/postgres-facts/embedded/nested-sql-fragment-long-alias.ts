import { query, sql } from "@example/db";

// More than eight aliases must still be recognized as a spliced fragment.
const fragment = sql`WHERE id = ${id}`;
const a1 = fragment;
const a2 = a1;
const a3 = a2;
const a4 = a3;
const a5 = a4;
const a6 = a5;
const a7 = a6;
const a8 = a7;
const a9 = a8;
const a10 = a9;
const a11 = a10;
const a12 = a11;
query(sql`SELECT id FROM accounts ${a12}`);

// Alias cycles terminate, including cycles with a fragment assignment seed.
let cycleA = cycleB;
let cycleB = cycleA;
cycleA = sql`WHERE id = ${id}`;
query(sql`SELECT id FROM accounts ${cycleB}`);
