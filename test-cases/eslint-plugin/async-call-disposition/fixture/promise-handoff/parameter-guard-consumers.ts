import type { Dependencies } from './types';
export async function parameterConsumer(unproven: any, dependencies: Dependencies, safe: Dependencies) {
  unproven(0, 0, dependencies);
  await safe.x([], undefined);
}
