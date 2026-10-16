import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// Preserve the earliest root-write phase and earlier receiver acquisitions.
(sink.value as any) = 1;
box.client.set = replacement;
function consume(value) { value.get('key'); }
consume(client);
