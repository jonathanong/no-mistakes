import { primary as client } from '@vendor/valkey';
function replacement() {}
function consume(value) {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
// consume executes only after its argument's destructuring defaults finish.
consume(({ first: unused = (holder.box.client = {}), second: unused = (holder.box.client.get = replacement) } = {}));
function globalPhaseArgumentBeforeCall(value) { value.get('key'); }
globalPhaseArgumentBeforeCall(client);
