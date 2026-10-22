import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
class Writer {
  mutate(_first, second = spare) { second.client.get = replacement; }
}
const writer = new Writer();
const args = [spare];
if (flag) { args.push({ client: replica }); }
// The supported prototype method call consumes writer as thisArg.
writer.mutate.call(writer, ...args, { client: untouched });
replica.get('method-call spread can occupy the second slot');
untouched.get('method-call suffix can occupy the second slot');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
