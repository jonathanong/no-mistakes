import { primary } from '@vendor/valkey';
function replacement() {}
// The nested callable retains its own execution context.
export default (() => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })())();
