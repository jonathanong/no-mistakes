import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  // This unconditional rebind excludes the original argument from the later write.
  value = box;
  value.client.get = replacement;
}
box = { client: replica };
mutate({ client: untouched });
replica.get('changed');
untouched.get('argument');
primary.get('old');
