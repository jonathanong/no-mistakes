import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = Object.assign({}, { client: replica });
function write() { box.client.get = replacement; }
write();
replica.get('mutated');
primary.get('control');
unrelated.get('unrelated');
// These assigns are not literal merges, so the binding stays a call.
const sink = {};
const namedTarget = Object.assign(sink, { extra: 1 });
const computedTarget = Object.assign({ ['extra']: 1 }, { extra: 2 });
const idSource = Object.assign({}, sink);
const computedSource = Object.assign({}, { ['extra']: sink });
const noArgs = Object.assign();
const spreadTarget = Object.assign(...[{}]);
const spreadSource = Object.assign({}, ...[{ extra: 1 }]);
const hole = Object.assign(sink, ...[,]);
const bareSpread = Object.assign(sink, ...sink);
void namedTarget;
void computedTarget;
void idSource;
void computedSource;
void noArgs;
void spreadTarget;
void spreadSource;
void hole;
void bareSpread;
