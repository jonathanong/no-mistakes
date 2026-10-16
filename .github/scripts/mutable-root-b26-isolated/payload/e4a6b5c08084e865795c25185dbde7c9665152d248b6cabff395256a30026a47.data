import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
box.client.set = replacement;
function consume(value) { value.get('key'); }
for (const item of ['a', 'b']) consume(client);
