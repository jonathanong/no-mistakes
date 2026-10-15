import { primary } from '@vendor/valkey';
function replacement() {}
export default { Client: class { accessor value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
