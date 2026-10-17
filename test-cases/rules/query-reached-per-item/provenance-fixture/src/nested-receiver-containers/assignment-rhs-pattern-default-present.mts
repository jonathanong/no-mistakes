import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: replica };
// A present value never acquires the default receiver.
[box = { client: replica }] = [{ client: primary }];
box.client.get = replacement;
function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
