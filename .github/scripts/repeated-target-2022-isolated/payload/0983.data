import { sharedReceiver, rhsCopiedReceiver, unrelatedReceiver, rhsSource, rhs } from '@vendor/producer';
for (const item of ['a', 'b']) {
  sharedReceiver.get(item);
  sharedReceiver.get = rhs.get;
  sharedReceiver.set(item, 'value');
}
for (const item of ['a', 'b']) {
  rhsCopiedReceiver.get(item);
  rhsCopiedReceiver.get = rhsSource.get;
}
for (const item of ['a', 'b']) {
  unrelatedReceiver.get(item);
  unrelatedReceiver.get = rhsSource.set;
}
