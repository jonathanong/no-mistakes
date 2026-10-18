import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function rebind(client) { box.client = client; }
function write() { box.client.get = replacement; }
[replica].forEach(rebind); write();
// A callback with no bare first parameter has no argument slot to project.
[replica].forEach(() => {});
// The first slot is destructured, so the later bare slot cannot supply it.
[replica].forEach(({}, later) => { later.info('other slot'); });
// With no callback argument there is no parameter origin to record.
[].forEach();
const candidates = [replica];
// A named array is outside the literal-array callback projection.
candidates.forEach(rebind);
replica.get('mutated');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
