import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const saved = client.get;
box.client.get = replacement;
function consume(fn) { fn('key'); }
consume(saved);
