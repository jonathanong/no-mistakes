import { primary } from '@vendor/valkey';
function replacement() {}
export default (() => { return () => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); })();
