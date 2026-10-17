import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
export default { [member]: function () { ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); } };
