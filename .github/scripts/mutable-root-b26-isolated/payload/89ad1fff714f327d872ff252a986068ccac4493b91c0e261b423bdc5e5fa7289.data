import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
box = box;
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
