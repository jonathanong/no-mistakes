import app from '../app.mts';
import '../legacy.js';
import { service0 } from '../services/service-0.mts';
import { clientCall0 } from '@fixture/http/client-0.mts';
import { enqueue0_0 } from '../producers/producer-0.mts';
import { enqueue1_1 } from '../producers/producer-1.mts';
import '../workers/worker-0.mts';
import '../workers/worker-1.mts';
export async function publicApiEntry() {
  await import('../routes/resource-0.mts');
  await clientCall0();
  enqueue0_0();
  enqueue1_1();
  return service0('entry');
}
app.get('/api/v1/entry/:id', () => publicApiEntry());
