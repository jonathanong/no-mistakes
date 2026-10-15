import { primary } from '@vendor/valkey';
function replacement() {}
export default (async (client = primary) => { const get = client.get; await Promise.resolve(); primary.get = replacement; get(); })();
