import { primary } from '@vendor/valkey';
function replacement() {}
export default (() => { for (let n = 0; n < 2; n++) { ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } })();
