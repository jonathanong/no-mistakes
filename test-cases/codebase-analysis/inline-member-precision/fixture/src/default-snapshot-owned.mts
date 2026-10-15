import { primary } from '@vendor/valkey';
function replacement() {}
// Default ownership does not defer evaluation of this expression.
export default ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
