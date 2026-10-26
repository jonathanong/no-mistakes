import type { Dependencies } from './types';

export async function run(dependencies: Dependencies) {
  await dependencies.x([], undefined);
}
