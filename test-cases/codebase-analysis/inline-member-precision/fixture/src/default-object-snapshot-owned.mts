import { primary } from '@vendor/valkey';
function replacement() {}
// Deferred siblings must restore module evaluation before the value initializer.
export default { deferred: () => undefined, method() {}, value: ((client = primary) => { const get = client.get; primary.get = replacement; get(); })() };
