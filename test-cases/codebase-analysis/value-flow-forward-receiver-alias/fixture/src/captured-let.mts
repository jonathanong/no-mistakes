import { primary, replica, untouched } from '@vendor/forward-captured-let';
function replacement() {}
let box = { client: primary };
function write(x) {
  let b = x;
  return () => { b.client.get = replacement; };
}
const f = write(box);
box = { client: replica };
f();
primary.get('changed');
// The saved parameter alias is conservatively Unknown across the later rebind.
replica.get('new');
untouched.get('control');
