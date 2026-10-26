// Configured alias misses are local uncertainty; ordinary npm imports are not.
import { missing } from '@app/missing';
import { external } from 'unrelated-external-package';
export { missing };
export const result = external;
