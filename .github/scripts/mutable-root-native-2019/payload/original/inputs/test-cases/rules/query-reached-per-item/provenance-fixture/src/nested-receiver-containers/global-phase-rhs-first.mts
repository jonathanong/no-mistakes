import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
// Destructuring evaluates its RHS before the first default assignment.
({ first: unused = (holder.box.client.get = replacement), second: holder.other } = (holder.box = { client }, {}));
function globalPhaseRhsFirst(value) { value.get('key'); }
globalPhaseRhsFirst(client);
