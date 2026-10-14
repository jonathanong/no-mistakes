import { primary } from '@vendor/valkey';
function replacement() {}
// Object construction does not execute this method.
export default { run() { ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
