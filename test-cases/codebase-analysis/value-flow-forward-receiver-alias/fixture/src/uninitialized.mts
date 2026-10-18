import { primary } from '@vendor/forward-uninitialized';
function replacement() {}
function write() { missing.get = replacement; }
let missing;
try { write(); } catch {}
function consume(value) { value.get('unrelated'); }
consume(primary);
