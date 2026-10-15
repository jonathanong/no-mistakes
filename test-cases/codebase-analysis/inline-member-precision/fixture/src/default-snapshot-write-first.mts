import { primary } from '@vendor/valkey';
function replacement() {}
export default ((client = primary) => { primary.get = replacement; const get = client.get; get(); })();
