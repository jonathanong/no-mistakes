import { primary, replica, untouched } from '@vendor/forward-loop-let';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
for (const client of [replica]) {
  box = { client };
  write();
}
replica.get('changed');
untouched.get('control');
