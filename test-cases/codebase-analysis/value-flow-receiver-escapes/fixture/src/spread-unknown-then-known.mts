import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(a, b, x) {
  x.client.get = replacement;
}
declare const rest: any[];
box = { client: replica };
// A literal spread after an unresolved spread still contributes its elements.
mutate(...rest, ...[box]);
replica.get('changed');
unrelated.get('control');
