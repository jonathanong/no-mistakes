import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
// `box` is the nested spread's element. A later element must not replace it.
mutate(...[...[box], other]);
replica.get('changed');
unrelated.get('control');
