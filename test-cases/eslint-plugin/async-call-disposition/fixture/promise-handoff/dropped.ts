import type { Dependencies } from './types';

export function run(dependencies: Dependencies) {
  dependencies.x([], undefined);
}
