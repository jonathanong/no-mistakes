import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
// A destructuring replacement must not mutate the previous root.
[box] = [{ client: replica }];
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
