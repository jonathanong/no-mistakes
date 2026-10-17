import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const saved = { box };
box = saved;
saved.box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
