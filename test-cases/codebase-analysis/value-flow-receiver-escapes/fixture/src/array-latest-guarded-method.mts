import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
class Writer {
  mutate(value) { value.client.get = replacement; }
  inspect(value) { value.client.info = replacement; }
}
const writer = new Writer();
const alias = writer;
let args = [box];
if (flag) { args = [other]; }
writer.mutate(...args);
alias.inspect(...args);
replica.get('earlier method branch');
untouched.get('latest method branch');
replica.info('earlier receiver-alias branch');
untouched.info('latest receiver-alias branch');
primary.get('unrelated origin');
unrelated.get('control');
