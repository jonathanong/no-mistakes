import { receiver } from '@vendor/scaling';
function replacement(key: string) { return key; }
for (const item of ['a', 'b']) {
  receiver.get(item); // physical read 1
  receiver.get(item); // physical read 2
  receiver.get(item); // physical read 3
  receiver.get(item); // physical read 4
  receiver.get(item); // physical read 5
  receiver.get(item); // physical read 6
  receiver.get(item); // physical read 7
  receiver.get(item); // physical read 8
  receiver.get(item); // physical read 9
  receiver.get(item); // physical read 10
  receiver.get(item); // physical read 11
  receiver.get(item); // physical read 12
  receiver.get(item); // physical read 13
  receiver.get(item); // physical read 14
  receiver.get(item); // physical read 15
  receiver.get(item); // physical read 16
  receiver.get = replacement;
}
