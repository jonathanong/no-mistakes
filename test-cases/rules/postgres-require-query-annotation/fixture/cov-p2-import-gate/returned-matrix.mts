import sql from 'sql-template-strings';
import {read} from '@bench/db';
import {returned} from './producer.mjs';
function safe() { const q = returned(sql`/* safe */ SELECT 1`); read(q.text, q.values); } // low: true
function typeOnly() { const q = returned(sql`/* typeOnly */ SELECT 1`); type Result = typeof q; read(q.text, q.values); } // low: true
export const exportedQuery = returned(sql`/* exported */ SELECT 1`);
read(exportedQuery.text, exportedQuery.values); // low: false
function repeated() { for (const id of ids) { const q = returned(sql`/* repeated */ SELECT ${id}`); read(q.text, q.values); } } // low: false
function captured() { const q = returned(sql`/* captured */ SELECT 1`); schedule(() => read(q.text, q.values)); } // low: false
function reused() { const q = returned(sql`/* reused */ SELECT 1`);
read(q.text, q.values); // low: false
read(q.text, q.values); } // low: false
function changed() { let q = returned(sql`/* changed */ SELECT 1`); q = other; read(q.text, q.values); } // low: false
function escaped() { const q = returned(sql`/* escaped */ SELECT 1`); schedule(() => opaque(q)); read(q.text, q.values); } // low: false
