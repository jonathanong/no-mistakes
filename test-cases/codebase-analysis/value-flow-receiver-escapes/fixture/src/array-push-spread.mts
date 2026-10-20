import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
const args = [];
// Calls in this window are not elements of args.
replacement();
unrelated.get('between');
const items = [];
items.push(box);
[].push(box);
args.push(box);
mutate(...args);
function late() {
  // The const is declared after this spread, so it is not an element yet.
  mutate(...args);
  const args = [box];
}
replica.get('changed');
unrelated.get('control');
