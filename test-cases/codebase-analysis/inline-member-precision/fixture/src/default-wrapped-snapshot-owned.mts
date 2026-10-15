import { primary } from '@vendor/valkey';
function replacement() {}
export default ({ value: ((client = primary) => { const get = client.get; primary.get = replacement; get(); })() } satisfies Record<string, unknown>);
