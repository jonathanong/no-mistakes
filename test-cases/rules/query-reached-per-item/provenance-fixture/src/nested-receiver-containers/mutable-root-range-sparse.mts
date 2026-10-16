import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
let other = { client };
// Interleaved leaves and conditional Either nodes require the exact Vec fallback.
box = { client };
other = { client };
if (condition) box = { client };
function mutate() { box.client.get = replacement; }
mutate();
client.get('key');
