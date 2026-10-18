import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// Every iteration can supply the receiver later mutated by write().
for (const client of [primary, replica]) {
  box.client = client;
  write();
}
primary.get('possibly mutated');
replica.get('possibly mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
