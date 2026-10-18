import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function rebind(client) {
  // The direct parameter writer and escaped member provider share call origins.
  client.get = replacement;
  box.client = client;
}
function write() { box.client.get = replacement; }
rebind(replica); write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
