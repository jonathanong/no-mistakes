import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// Dynamic names do not make callable bodies module evaluation.
export default { [member]: () => ((client = primary) => { const get = client.get; primary.get = replacement; get(); })() };
