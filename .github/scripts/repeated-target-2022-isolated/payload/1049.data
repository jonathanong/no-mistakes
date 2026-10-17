import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// Default parameters execute when the computed method is called.
export default { [member](value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })()) {} };
