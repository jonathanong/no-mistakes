import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
function invoke(fn, box, other) {
  const args = [spare];
  if (flag) { args.push(box); }
  // The suffix is the second argument when the optional receiver is absent.
  fn(...args, other);
}
invoke(mutate, { client: replica }, { client: untouched });
replica.get('forwarded optional push can occupy the second slot');
untouched.get('forwarded suffix can occupy the second slot');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
