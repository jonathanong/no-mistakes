import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// Class computed keys execute before entering their callable bodies.
export default { Client: class { [((client = primary) => { const get = client.get; primary.get = replacement; get(); return 'read'; })()]() {} } };
