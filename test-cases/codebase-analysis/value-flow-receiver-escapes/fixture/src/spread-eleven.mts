import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, x) {
  x.client.get = replacement;
}
box = { client: replica };
// Element 10 is a numeric index, not the text key "10" sorted before "2".
mutate(...[other, other, other, other, other, other, other, other, other, other, box]);
replica.get('changed');
unrelated.get('control');
