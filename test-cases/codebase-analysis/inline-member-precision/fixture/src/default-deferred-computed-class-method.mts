import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
export default { Client: class { [key]() { return ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } } };
