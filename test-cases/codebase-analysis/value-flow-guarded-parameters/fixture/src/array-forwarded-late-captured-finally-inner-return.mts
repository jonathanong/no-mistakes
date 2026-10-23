import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const a = { client: replica };
const b = { client: untouched };
function mutate(box) {
  const captured = [box];
  box = { client: primary };
  function write(value) { value.client.get = replacement; }
  write(...captured);
}
function invoke(value, callback) {
  callback = mutate;
  try { return; } finally {
    if (flag) return;
    callback(value);
  }
}
// flag=true returns from finally before writing and reaches untouched.get.
declare const flag: boolean;
invoke(b);
mutate(a);
replica.get('captured direct origin was replaced');
untouched.get('late forwarded origin was replaced');
primary.get('rebound local control');
replica.info('sibling method control');
unrelated.get('unrelated receiver control');
