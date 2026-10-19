import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(a, x) {
  x.client.get = replacement;
}
box = { client: replica };
// The nested conditional is parameter 1, so both arms can be `x`.
mutate(...[other, ...[flag ? other : box]]);
replica.get('changed');
unrelated.get('control');
