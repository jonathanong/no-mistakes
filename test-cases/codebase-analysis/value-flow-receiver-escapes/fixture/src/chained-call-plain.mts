import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write(x) {
  x.client.get = replacement;
}
box = { client: replica };
// No spread on `write`, so its call stores no extras. `.later` does.
write({ client: unrelated }).later(...[box]);
replica.get('changed');
unrelated.get('control');
primary.get('old');
