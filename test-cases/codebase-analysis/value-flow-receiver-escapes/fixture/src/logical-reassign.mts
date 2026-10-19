import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  // A nullish argument takes the fallback. `??=` is still a write of `box`.
  value ??= box;
  value.client.get = replacement;
}
box = { client: replica };
mutate(null);
replica.get('changed');
unrelated.get('control');
primary.get('old');
