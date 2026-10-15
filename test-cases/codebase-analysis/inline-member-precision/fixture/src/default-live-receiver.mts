import { primary } from '@vendor/valkey';
function replacement() {}
export default ((client = primary) => { const alias = client; primary.get = replacement; alias.get(); })();
