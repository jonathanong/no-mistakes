import { insertImpact } from '../test-helpers/insert.mts';
import { lockReferences } from '../test-helpers/lock.mts';

export async function exercise() {
  await insertImpact('a', '1');
  await lockReferences(['posts', 'users']);
}
