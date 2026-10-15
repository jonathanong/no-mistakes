import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
export default { Client: class { [((client = primary) => { const get = client.get; primary.get = replacement; get(); })()]() {} } };
