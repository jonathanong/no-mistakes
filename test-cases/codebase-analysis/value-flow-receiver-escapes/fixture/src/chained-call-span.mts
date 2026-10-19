import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write(x) {
  x.client.get = replacement;
}
box = { client: replica };
// The inner call and `.later` share a start offset. `box` is the inner argument.
write(...[box]).later(...[unrelated]);
replica.get('changed');
unrelated.get('control');
