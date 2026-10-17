import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// The computed key executes while constructing the exported object.
export default { [((client = primary) => { const get = client.get; primary.get = replacement; get(); return 'read'; })()]: () => {} };
