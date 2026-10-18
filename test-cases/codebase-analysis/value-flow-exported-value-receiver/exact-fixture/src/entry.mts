import { receiver } from './receiver.mts';
import { acquire } from './helper.mts';
import { rhs } from '@vendor/exported-value';
for (const item of ['a', 'b']) {
  const method = acquire();
  method(item);
  receiver.get = rhs.get;
}
