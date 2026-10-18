import { receiver } from './receiver.mts';
import { acquire, acquireSibling, acquireUnrelated } from './helper.mts';
import { rhs } from '@vendor/exported-value';
const copied = acquire();
for (const item of ['a', 'b']) {
  const method = acquire();
  method(item);
  const sibling = acquireSibling();
  sibling(item);
  const other = acquireUnrelated();
  other(item);
  copied(item);
  receiver.get = rhs.get;
}
