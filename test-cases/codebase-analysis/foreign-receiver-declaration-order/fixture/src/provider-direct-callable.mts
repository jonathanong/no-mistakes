import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// A direct arrow invocation supplies a callable owner without a named callee.
((client) => { box.client = client; })(replica);
write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
