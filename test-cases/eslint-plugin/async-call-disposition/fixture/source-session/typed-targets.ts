import { enqueue } from '@session/jobs';
import { factory } from './shared/factory';
type Job = typeof enqueue;
const typed: Job = enqueue;
typed('alias');
factory().run('factory');
export function invoke(deps: { run: typeof enqueue }) {
  deps.run('parameter');
}
