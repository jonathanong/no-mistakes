import type { Dependencies } from './types';
export function staticFinal(dependencies: Dependencies) {
  dependencies['x']([], undefined);
}
