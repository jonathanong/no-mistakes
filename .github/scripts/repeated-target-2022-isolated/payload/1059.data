import { primary } from '@vendor/valkey';
function replacement() {}
// A parenthesized exported function remains deferred.
export default (function run() { ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); });
