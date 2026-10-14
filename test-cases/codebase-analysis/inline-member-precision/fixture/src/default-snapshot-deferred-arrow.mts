import { primary } from '@vendor/valkey';
function replacement() {}
// The exported arrow can run repeatedly.
export default () => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
