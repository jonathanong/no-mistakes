import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function outer(first, second) {
  const write = () => { arguments[1].client.get = replacement; };
  write();
}
box = { client: replica };
outer(other, box);
replica.get('changed');
untouched.get('first');
unrelated.get('control');
