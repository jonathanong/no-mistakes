import { primary } from '@vendor/valkey';
function replacement() {}
export default function run() { return ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(); }
