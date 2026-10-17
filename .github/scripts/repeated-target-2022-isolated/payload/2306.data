import { onceReceiver, unsupportedReceiver, nestedReceiver, dynamicReceiver, resetReceiver, leftResetReceiver, destructuredReceiver, rhs } from '@vendor/producer';
declare const condition: boolean;
declare const key: string;
declare const opaque: (...args: unknown[]) => unknown;
for (const item of ['a']) {
  onceReceiver.get(item);
  onceReceiver.get = rhs.get;
}
while (condition) {
  unsupportedReceiver.get('key');
  unsupportedReceiver.get = rhs.get;
}
for (const item of ['a', 'b']) {
  nestedReceiver.get(item);
  for (const child of ['c', 'd']) { nestedReceiver.get = rhs.get; }
}
// A dynamic member write is overwritten; its unknown value is not the tail.
for (const item of ['a', 'b']) {
  dynamicReceiver.get(item);
  dynamicReceiver[key] = opaque;
  dynamicReceiver.get = rhs.get;
}
// Imported binding reassignment cannot prove receiver persistence.
for (const item of ['a', 'b']) {
  resetReceiver = rhs;
  resetReceiver.get(item);
  resetReceiver.get = rhs.get;
}

// Loop-left assignment resets the binding before every unsupported iteration.
for (leftResetReceiver of [rhs, rhs]) {
  leftResetReceiver.get('key');
}
for (const item of ['a', 'b']) {
  destructuredReceiver.get(item);
  [destructuredReceiver] = [rhs];
}
