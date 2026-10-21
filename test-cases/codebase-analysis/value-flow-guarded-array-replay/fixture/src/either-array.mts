import { replica, tail, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const late = { client: tail };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
// Array element branches share slot zero, unlike an opaque conditional push.
let args = [flag ? box : late, other];
if (flag) { args = [other]; }
mutate(...args);
replica.get('left array-element branch');
tail.get('right array-element branch');
untouched.get('historical latest origin');
replica.info('sibling control');
unrelated.get('control');
