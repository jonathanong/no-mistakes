import { primary } from '@vendor/valkey';
function replacement() {}
// Suspension prevents an immediate invocation proof.
export default (async (client = primary) => { await Promise.resolve(); const get = client.get; primary.get = replacement; get(); })();
