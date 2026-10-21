import { replica, tail, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const late = { client: tail };
const other = { client: untouched };
function mutate(first, second, third, fourth) {
  first.client.get = replacement;
  second.client.get = replacement;
  third.client.get = replacement;
  fourth.client.get = replacement;
}
let args = [box];
// The runtime guards correlate array width and tail position, but the saved
// projection tracks them independently and may conservatively admit slot 5.
args.push(late);
if (flag) { args = [,,other]; }
// Conditional push arguments stay Unknown; sparse width is max slot plus one.
args.push(flag ? late : box);
if (flag) { args = [other]; }
args.push(late);
mutate(...args);
replica.get('older array and conditional push');
tail.get('historical latest pushed origin');
untouched.get('historical latest array origin');
replica.info('sibling control');
unrelated.get('control');
