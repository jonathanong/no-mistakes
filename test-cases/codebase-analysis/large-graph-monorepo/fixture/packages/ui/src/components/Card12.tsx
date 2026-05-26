import { coreFn12 } from '@fixture/core/core-12.mts';
import type { DataRecord12 } from '@fixture/data/records/data-12.mts';
export function Card12({ record }: { record: DataRecord12 }) {
  return <section data-testid="card-12">{coreFn12()}{record.id}</section>;
}
