import { write } from '@app/db';
import { imported } from './shared.mjs';
function shadow(arguments: unknown) { const slots = arguments; delete slots[0]; return () => write('SELECT 401'); }
function rebound(sql: string) { let arguments = sql; return () => write('SELECT 402'); }
function opaque(sql: string) { unknownConsumer(sql); return () => write('SELECT 403'); }
function globalAlias(sql: string) { const slots = external; delete slots[0]; return () => write('SELECT 404'); }
function receiver(sql: string) { const slots = arguments; slots.member = sql; return () => write('SELECT 405'); }
function dead(sql: string) { if (false) unknownConsumer(sql); return () => write('SELECT 406'); }
function require(sql: string) { const slots = arguments; return () => write(slots[0]); }
// Rejection concerns executed factory state, not the deferred closure's effects.
const a = flag ? shadow('SELECT 411') : shadow('SELECT 412');
const b = flag ? rebound('SELECT 421') : rebound('SELECT 422');
const c = flag ? opaque('SELECT 431') : opaque('SELECT 432');
const d = flag ? globalAlias('SELECT 441') : globalAlias('SELECT 442');
const e = flag ? receiver('SELECT 451') : receiver('SELECT 452');
const f = flag ? dead('SELECT 461') : dead('SELECT 462');
const g = flag ? require('SELECT 471') : require('SELECT 472');
const h = flag ? imported('SELECT 481') : imported('SELECT 482');

function aliasThis(sql: string) { const slots = this; delete slots[0]; return () => write('SELECT 491'); }
function aliasRebind(sql: string) { let slots = arguments; slots = external; return () => write('SELECT 492'); }
function reserved(sql: string) { let slots; return () => write('SELECT 493'); }
function executor(sql: string) { write(sql); return () => write('SELECT 494'); }
function moduleLoad(sql: string) { const slots = require('./shared.mjs'); return () => write('SELECT 495'); }
const i = flag ? aliasThis('SELECT 501') : aliasThis('SELECT 502');
const j = flag ? aliasRebind('SELECT 511') : aliasRebind('SELECT 512');
const k = flag ? reserved('SELECT 521') : reserved('SELECT 522');
const l = flag ? executor('SELECT 531') : executor('SELECT 532');
const m = flag ? moduleLoad('SELECT 541') : moduleLoad('SELECT 542');

// Ordinary functions should fail the constant-time return-shape precheck.
function ordinary(sql: string) { return sql; }
const n = flag ? ordinary('SELECT 551') : ordinary('SELECT 552');

function twice(sql: string) { return () => write(sql); return () => write(sql); }
const o = flag ? twice('SELECT 561') : twice('SELECT 562');
const p = flag ? obj.pack('SELECT 571') : obj.pack('SELECT 572');
