import { once, exclusive, abrupt, replica } from '@vendor/valkey';
// A known single iteration cannot observe its tail on another iteration.
for (const item of ['a']) {
  once.get(item);
  once.get = replica.get;
}
function branch() {
  exclusive.get('key');
  exclusive.get = replica.get;
}
declare const flag: boolean;
// Mutually exclusive source calls do not establish repeated execution.
if (flag) { branch(); } else { branch(); }
for (const item of ['a', 'b']) {
  abrupt.get(item);
  abrupt.get = replica.get;
  break;
}
