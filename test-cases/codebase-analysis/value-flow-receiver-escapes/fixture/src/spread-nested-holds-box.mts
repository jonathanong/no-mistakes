import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
// The inner spread's only element is parameter 0. Dropping it leaves replica Known.
mutate(...[...[box]]);
replica.get('changed');
unrelated.get('control');
