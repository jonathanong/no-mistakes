import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function rebind(client) {
  box.client = client;
  outer(client);
}
function outer(value) { rebind(value.child); }
function write() { box.client.get = replacement; }
// The recursive branch grows `.child`; the concrete branch reaches replica.
outer({ child: replica }); write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
