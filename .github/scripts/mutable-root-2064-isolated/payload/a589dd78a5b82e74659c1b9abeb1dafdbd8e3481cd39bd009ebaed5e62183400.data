import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
({ first: unused = (holder.box.client = client), second: unused = (holder.box.client.get = replacement) } = {});
function globalPhaseOriginalRestore(value) { value.get('key'); }
globalPhaseOriginalRestore(client);
