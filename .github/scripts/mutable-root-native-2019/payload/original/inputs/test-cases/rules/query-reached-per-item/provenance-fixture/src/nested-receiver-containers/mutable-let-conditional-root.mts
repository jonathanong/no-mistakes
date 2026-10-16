import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) box = { client: replica };
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
