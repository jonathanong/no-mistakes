import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() { box.client.get = replacement; }
// Even an uncalled static provider remains a conservative callable summary.
function staticProvider() { box.client = primary; }
function read({ client }) { return client; }
const sibling = read({ client: unrelated });
// An excess argument must not become the arrow's receiver provider.
((client) => { box.client = client; })(replica, unrelated);
write();
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unused argument');
sibling.get('readonly destructuring remains known');
