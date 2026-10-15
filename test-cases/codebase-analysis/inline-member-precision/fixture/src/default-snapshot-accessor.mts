import { primary } from '@vendor/valkey';
function replacement() {}
// Auto-accessor initializers also run for every instance.
export default { Client: class { accessor value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
