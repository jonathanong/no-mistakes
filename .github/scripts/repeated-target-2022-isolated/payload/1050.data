import { primary } from '@vendor/valkey';
function replacement() {}
declare const member: string;
// Each computed key and the final value execute after the previous callable body traversal.
export default {
  [member]: () => (() => 0)(),
  [(() => 'function')()]: function () { (() => 0)(); },
  [(() => 'method')()]() { (() => 0)(); },
  value: ((client = primary) => { const get = client.get; primary.get = replacement; get(); })(),
};
