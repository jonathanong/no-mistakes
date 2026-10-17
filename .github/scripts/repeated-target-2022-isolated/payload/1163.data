import { nodeReceiver, rhs } from '@vendor/producer';
import { objectReceiver } from './repository-object.mts';
function replacement(key: string) { return key; }
for (const item of ['a', 'b']) {
  nodeReceiver.get(item);
  nodeReceiver.get = replacement;
}
// These identical canonical Call edges are separate physical occurrences.
replacement('one'); replacement('two');
const savedObject = objectReceiver.get;
for (const item of ['a', 'b']) {
  objectReceiver.get(item);
  objectReceiver.get = replacement;
  objectReceiver.set(item, 'value');
  savedObject(item);
}
