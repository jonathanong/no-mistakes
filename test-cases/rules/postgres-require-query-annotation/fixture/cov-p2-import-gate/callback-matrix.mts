import sql from 'sql-template-strings';
import {read} from '@bench/db';
import {page} from './callbacks.mjs';
function safe(q) { q.append(sql` LIMIT 1`); }
page(sql`/* safe */ SELECT 1`, q => { safe(q); return read(q); }); // low: true
function noParams() { q.append(sql` LIMIT 1`); }
page(sql`/* noParams */ SELECT 1`, q => { noParams(q); return read(q); }); // low: false
function extraArg(q) { q.append(sql` LIMIT 1`, extra); }
page(sql`/* extraArg */ SELECT 1`, q => { extraArg(q); return read(q); }); // low: false
function appendMany(q) { q.append(...parts); }
page(sql`/* spread */ SELECT 1`, q => { appendMany(q); return read(q); }); // low: false
function memberTag(q) { q.append(tags.sql` LIMIT 1`); }
page(sql`/* memberTag */ SELECT 1`, q => { memberTag(q); return read(q); }); // low: false
function computed(q) { q['append'](sql` LIMIT 1`); }
page(sql`/* computed */ SELECT 1`, q => { computed(q); return read(q); }); // low: false
function optionalReceiver(q) { q?.append(sql` LIMIT 1`); }
page(sql`/* optionalReceiver */ SELECT 1`, q => { optionalReceiver(q); return read(q); }); // low: false
function callReceiver(q) { getBuilder().append(sql` LIMIT 1`); }
page(sql`/* callReceiver */ SELECT 1`, q => { callReceiver(q); return read(q); }); // low: false
function wrongMethod(q) { q.prepend(sql` LIMIT 1`); }
page(sql`/* wrongMethod */ SELECT 1`, q => { wrongMethod(q); return read(q); }); // low: false
function bareBody(q) { void q; }
page(sql`/* bareBody */ SELECT 1`, q => { bareBody(q); return read(q); }); // low: false
