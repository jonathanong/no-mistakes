import { overwritten, prefix, middle, opaque, replica, tail, first } from '@vendor/valkey';
// A later write survives the backedge; an intermediate overwritten value does not.
for (const item of ['a', 'b']) {
  overwritten.get(item);
  overwritten.get = replica.get;
  overwritten.get = tail.get;
}
// A dominating prefix write overwrites even the prior iteration's tail.
for (const item of ['a', 'b']) {
  prefix.get = first.get;
  prefix.get(item);
  prefix.get = tail.get;
}
for (const item of ['a', 'b']) {
  middle.get(item);
  middle.get = first.get;
  middle.get(item);
  middle.get = tail.get;
}
declare const unknownMethod: (...args: unknown[]) => unknown;
for (const item of ['a', 'b']) {
  opaque.get(item);
  opaque.get = unknownMethod;
}
