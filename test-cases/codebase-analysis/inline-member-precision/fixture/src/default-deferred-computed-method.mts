import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
// A computed method remains deferred even without a static ownership name.
export default { [key]() { return ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
