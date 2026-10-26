import { enqueueEmail } from '@app/jobs';
import type { Dependencies } from './types';
import { staticFinal } from './static-final';
export function staticForward(dependencies: Dependencies) {
  return () => staticFinal({ x: dependencies.x ?? enqueueEmail });
}
