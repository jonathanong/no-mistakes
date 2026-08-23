import app from './app';
import './legacy.js';
import { service0 } from './services/service-0';
import { clientCall0 } from '@fixture/http/client-0';
import { enqueue0_0 } from './producers/producer-0';
import { enqueue1_1 } from './producers/producer-1';
import './workers/worker-0';
import './workers/worker-1';
export async function publicApiEntry() {
  await import('./routes/resource-0');
  await clientCall0();
  enqueue0_0();
  enqueue1_1();
  return service0('entry');
}
app.get('/api/v1/entry/:id', () => publicApiEntry());
