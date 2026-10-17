import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { box.client = {}; }
else { box.client.get = replacement; }
function consume(value) { value.get('key'); }
for (const item of ['a', 'b']) consume(client);
