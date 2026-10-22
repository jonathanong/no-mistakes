import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
function invoke(fn, box) {
  const args = [];
  if (flag) { args.push({ client: untouched }); }
  // The fixed spread and ordinary box precede the optional tail in the forwarded call.
  fn(...[spare], box, ...args);
}
invoke(mutate, { client: replica });
replica.get('forwarded fixed receiver was overwritten locally');
untouched.get('forwarded optional tail remains unrelated');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
