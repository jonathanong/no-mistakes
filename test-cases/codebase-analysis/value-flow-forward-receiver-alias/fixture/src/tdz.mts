import { primary } from '@vendor/forward-tdz';
function replacement() {}
function write() { alias.client.get = replacement; }
// The call cannot establish a replacement before the receiver exists.
try { write(); } catch {}
const alias = { client: primary };
function consume(value) { value.get('primary'); }
consume(primary);
