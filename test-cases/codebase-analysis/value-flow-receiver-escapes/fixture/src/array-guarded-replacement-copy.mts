import { primary, replica, untouched, destination, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
let args = [box];
if (flag) { args = [other]; }
mutate(...args);
// A copied uncertain RHS must retain its original executor in the payload.
for (const key of ['one', 'two']) {
  destination.get = replica.get;
  destination.get(key);
}
primary.get('control');
unrelated.get('control');
