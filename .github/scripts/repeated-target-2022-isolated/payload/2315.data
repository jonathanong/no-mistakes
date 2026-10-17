import { loopReceiver, prefixReceiver, middleReceiver, rhs, tail } from '@vendor/producer';
for (const item of ['a', 'b']) {
  loopReceiver.get(item);
  loopReceiver.get = rhs.get;
}
for (const item of ['a', 'b']) {
  prefixReceiver.get = rhs.get;
  prefixReceiver.get(item);
  prefixReceiver.get = tail.get;
}
for (const item of ['a', 'b']) {
  middleReceiver.get(item);
  middleReceiver.get = rhs.get;
  middleReceiver.get(item);
  middleReceiver.get = tail.get;
}
