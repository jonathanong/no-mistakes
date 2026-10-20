import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate() {
  const aliased = arguments;
  aliased[0].client.get = replacement;
}
box = { client: replica };
mutate(box);
replica.get('changed');
primary.get('old');
unrelated.get('control');
