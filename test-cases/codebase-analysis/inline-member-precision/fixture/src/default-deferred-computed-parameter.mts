import { primary } from '@vendor/valkey';
function replacement() {}
declare const key: string;
// Parameter expressions run on every invocation, before the method body.
export default { [key](value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })()) { return value; } };
