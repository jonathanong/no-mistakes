import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
// Only the last comma operand is passed. Both conditional leaves are receivers.
mutate((0, flag ? other : box));
replica.get('changed');
primary.get('old');
unrelated.get('control');
