import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
export default ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
