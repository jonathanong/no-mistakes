import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const retained = { client: replica };
box = retained;
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(replica);
