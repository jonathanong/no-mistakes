import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
// Conditional fresh replacement preserves the already established detachment.
({ first: unused = (holder.box.client = {}), second: unused = (holder.box.client.get = replacement) } = {});
function globalPhasePreservedFresh(value) { value.get('key'); }
globalPhasePreservedFresh(client);
