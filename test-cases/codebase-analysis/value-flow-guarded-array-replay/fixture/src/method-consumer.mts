import { replica, tail, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const late = { client: tail };
const other = { client: untouched };
class Writer {
  mutate(value) { value.client.get = replacement; }
}
const writer = new Writer();
let args = [flag ? box : late];
if (flag) { args = [other]; }
// The final recipient is the recorded instance method, not an ordinary callee.
writer.mutate(...args);
replica.get('left earlier array origin');
tail.get('right earlier array origin');
untouched.get('historical latest origin');
replica.info('sibling control');
unrelated.get('control');
