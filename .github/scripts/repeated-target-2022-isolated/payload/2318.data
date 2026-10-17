import { savedReceiver, returnedReceiver, outsideReceiver, rhs } from '@vendor/producer';
function acquire() { return returnedReceiver.get; }
// Module acquisition precedes the loop, even through an inline parameter.
const outside = outsideReceiver.get;
for (const item of ['a', 'b']) {
  const saved = savedReceiver.get;
  saved(item);
  savedReceiver.get = rhs.get;
}
for (const item of ['a', 'b']) {
  const returned = acquire();
  returned(item);
  returnedReceiver.get = rhs.get;
}
for (const item of ['a', 'b']) {
  ((method: (key: string) => unknown) => method(item))(outside);
  outsideReceiver.get = rhs.get;
}
