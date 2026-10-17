import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const captured = box;
box = {};
captured.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
