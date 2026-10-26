import { enqueueEmail } from '@app/jobs';
import type { Dependencies } from './types';
import { run } from './final';
import { run as runDropped } from './dropped';
// These handlers escape the builder, so the old immediate-await proof refuses them.
export function prepareCrawl(options: Dependencies) {
  const handler = () => {
    const dependencies = { x: options.x ?? enqueueEmail, urlId: options.urlId };
    return run(dependencies);
  };
  return handler;
}
export function prepareDropped(options: Dependencies) {
  const handler = () => {
    const dependencies = { x: options.x ?? enqueueEmail, urlId: options.urlId };
    return runDropped(dependencies);
  };
  return handler;
}
