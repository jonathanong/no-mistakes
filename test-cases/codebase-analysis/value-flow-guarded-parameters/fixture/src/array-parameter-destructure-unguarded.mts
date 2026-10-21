import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  [box] = [other];
  const args = [box];
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('the destructuring replacement never writes the original argument');
untouched.get('retain a previously reported candidate with unknown evidence');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
