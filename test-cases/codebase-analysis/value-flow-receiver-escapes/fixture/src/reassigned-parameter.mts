import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  // The original argument and this reassignment are both possible receivers.
  value = box;
  value.client.get = replacement;
}
box = { client: replica };
mutate({ client: untouched });
replica.get('changed');
untouched.get('argument');
primary.get('old');
