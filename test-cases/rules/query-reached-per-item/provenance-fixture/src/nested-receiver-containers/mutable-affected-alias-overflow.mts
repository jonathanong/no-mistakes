import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
const before = client.get;
// More than two aliases must promote without dropping a receiver name.
const first = { client };
const second = first;
const third = second;
third.client.get = replacement;
before('copied');
function consume(value: typeof client) { value.get('changed'); }
consume(client);
replica.get('untouched');
