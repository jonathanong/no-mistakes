import { closedOnceReceiver, unsupportedOnceReceiver, rhs } from '@vendor/producer';
// A literal-one body can still run again through its enclosing callable.
function closedRepeated() {
  for (const item of ['one']) {
    closedOnceReceiver.get(item);
    closedOnceReceiver.set(item, 'value');
    closedOnceReceiver.get = rhs.get;
  }
}
closedRepeated(); closedRepeated();
export function unsupportedOwner() {
  for (const item of ['one']) {
    unsupportedOnceReceiver.get(item);
    unsupportedOnceReceiver.get = rhs.get;
  }
}
