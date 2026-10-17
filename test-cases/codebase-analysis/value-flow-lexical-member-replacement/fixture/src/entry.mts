import { primary, replica, untouched, dynamic, opaque } from '@vendor/receiver';
const saved = primary.get;
primary.get('before');
primary.get = replica.get;
for (const item of ['a', 'b']) {
  // This one-time lexical write precedes every iteration.
  primary.get(item);
  saved(item);
  primary.info(item);
  untouched.get(item);
}
dynamic[key] = replica.get;
for (const item of ['a', 'b']) { dynamic.get(item); }
opaque.get = unknownMethod;
for (const item of ['a', 'b']) { opaque.get(item); }
