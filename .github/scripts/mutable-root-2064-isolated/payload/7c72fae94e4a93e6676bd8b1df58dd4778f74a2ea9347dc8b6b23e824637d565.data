import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
// TypeScript wrappers do not change which lexical root is replaced.
(box satisfies any) = { client: replica };
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(client);
