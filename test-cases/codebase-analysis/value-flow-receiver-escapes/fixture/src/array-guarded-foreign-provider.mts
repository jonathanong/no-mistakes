import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
function write() {
  let args = [box];
  if (flag) { args = [other]; }
  mutate(...args);
}
// This provider is visited after the forwarded writer's acquisition.
function rebind() { box = { client: replica }; }
rebind();
write();
primary.get('initial foreign provider stays possible');
replica.get('later foreign provider stays possible');
untouched.get('historical later array origin');
replica.info('sibling member stays known');
unrelated.get('control');
