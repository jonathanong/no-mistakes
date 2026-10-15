import { primary } from '@vendor/valkey';
function replacement() {}
// Exporting a callable does not make its body module-evaluated.
export default () => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
