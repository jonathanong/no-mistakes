import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// No formal: the index is the only slot. The named function is the formal slot.
function mutate() {
  arguments = replacement;
  arguments[box] = replacement;
  arguments.client = replacement;
  arguments[0].client.get = replacement;
}
function mutateNamed(value) { arguments[0].client.get = replacement; }
box = { client: replica };
mutate(box);
mutateNamed(box);
replica.get('changed');
primary.get('old');
untouched.get('control');
arguments[0].client.get = replacement;
