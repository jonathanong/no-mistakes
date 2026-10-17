import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
export default { Client: class { [member]() { ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } } };
