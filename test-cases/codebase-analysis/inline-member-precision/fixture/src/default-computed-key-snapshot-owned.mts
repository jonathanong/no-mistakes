import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
// The key evaluates eagerly before entering the deferred method.
export default { [((client = primary) => { const get = client.get; primary.get = replacement; get(); })()]() {} };
