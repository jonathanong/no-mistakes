import { primary } from '@vendor/valkey';
function replacement() {}
// An anonymous class field runs for every constructed instance.
export default { Client: class { value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
